use anyhow::Context;
use async_session::base64;
use axum::{
    extract::{Query, State},
    http::{header::SET_COOKIE, HeaderMap},
    response::{IntoResponse, Redirect},
};
use axum_extra::{extract::cookie::Cookie, headers, TypedHeader};
use rand::RngCore;
use serde::Deserialize;

use crate::{error::{app_error::AppError, token_error::TokenError}, repository::session_repository::SessionRepositoryTrait, service::google_token_service::{GoogleTokenService, TokenServiceTrait}, AppState};

static SESSION_COOKIE_NAME: &str = "SESSION";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
    code: String,
    state: String,
}

pub async fn google_auth(
    State(app_state): State<AppState>,
    State(google_token_service): State<GoogleTokenService>,
) -> Result<impl IntoResponse, AppError> {
    let (auth_url, csrf_token) = google_token_service.generate_authorisation_url().await?;

    let session_id = generate_session_id();
    app_state.session_repository.add_csrf_token(&session_id, csrf_token.secret()).await?;

    let cookies = [format!(
        "{SESSION_COOKIE_NAME}={session_id}; SameSite=Lax; HttpOnly; Secure; Path=/"
    )];
    let mut headers = HeaderMap::new();
    for cookie in cookies {
        headers.append(
            SET_COOKIE,
            cookie.parse().context("Failed to parse header value")?,
        );
    }

    Ok((headers, Redirect::to(auth_url.as_ref())))
}

pub async fn auth_callback(
    Query(query): Query<AuthRequest>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    State(app_state): State<AppState>,
    State(google_token_service): State<GoogleTokenService>,
) -> Result<impl IntoResponse, AppError> {
    tracing::debug!("Handling google auth callback");
    validate_csrf_token(&app_state, &query, &cookies).await?;

    let (access_token, refresh_token) = google_token_service.exchange_authorisation_code(query.code.clone()).await?;

    let access_token = access_token.secret().to_string();

    let refresh_token = refresh_token.secret().to_string();

    let user_data = google_token_service.get_user_info(&access_token).await?;

    let user_context = app_state.user_service.find_or_insert_user(&user_data).await?;
    app_state.set_user_context(user_context).await;

    let cookies = [
        format!("access_token={access_token}; SameSite=Lax; HttpOnly; Secure; Path=/"),
        format!("refresh_token={refresh_token}; SameSite=Lax; HttpOnly; Secure; Path=/"),
    ];
    let mut headers = HeaderMap::new();
    for cookie in cookies {
        headers.append(
            SET_COOKIE,
            cookie.parse().context("Failed to parse header value")?,
        );
    }

    Ok((headers, Redirect::to("/")))
}

async fn validate_csrf_token(
    app_state: &AppState,
    auth_request: &AuthRequest,
    cookies: &headers::Cookie,
) -> Result<(), AppError> {
    tracing::debug!("Validating CSRF token for google auth callback");
    let session_id = cookies
        .get(SESSION_COOKIE_NAME)
        .context("Unexpected error getting cookie name")?
        .to_string();

    let stored_csrf_token = app_state.session_repository.get_csrf_token_by_session_id(&session_id).await?;
    app_state.session_repository.expire_session(&session_id).await?;

    if stored_csrf_token != auth_request.state {
        return Err(TokenError::GenericTokenError("CSRF token mismatch".to_string()).into());
    }

    Ok(())
}

fn generate_session_id() -> String {
    let mut key = vec![0u8; 64];
    rand::thread_rng().fill_bytes(&mut key);
    base64::encode(key)
}

pub async fn logout(
    State(app_state): State<AppState>,
    TypedHeader(cookies): TypedHeader<headers::Cookie>,
    State(google_token_service): State<GoogleTokenService>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(user_id) = app_state.get_user_id().await {
        tracing::debug!("Logging out user with ID: {}", user_id);
    }
    // TODO: Revocation of access and refresh token not necessary as revoking a refresh 
    // token in Google OAUTH 2.0 also revokes the associated access token and vice versa.
    // See: https://cloud.google.com/apigee/docs/api-platform/security/oauth/validating-and-invalidating-access-tokens
    if let Some(refresh_token) = cookies.get("refresh_token") {
        google_token_service.revoke_token(refresh_token.to_string()).await?;
    }

    app_state.clear_user_context().await;

    // TODO - refactor
    let empty_access_token = Cookie::build(("access_token", ""))
        .path("/")
        .http_only(true)
        .build();
    let empty_refresh_token = Cookie::build(("refresh_token", ""))
        .path("/")
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();
    headers.append(
        SET_COOKIE,
        empty_access_token.to_string().parse().unwrap(),
    );
    headers.append(
        SET_COOKIE,
        empty_refresh_token.to_string().parse().unwrap(),
    );

    Ok((headers, Redirect::to("/")))
}

#[cfg(test)]
mod tests {

    use axum_extra::headers::{Cookie, HeaderMapExt};
    use chrono::Utc;
    use http::HeaderMap;
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, TokenUrl};
    use sqlx::MySqlPool;

    use crate::{assert_error, config::database::Database, error::{app_error::AppError, token_error::TokenError}, handler::auth_handler::validate_csrf_token, repository::session_repository::{SessionRepository, SessionRepositoryTrait}, state::app_state::AppState};


    async fn setup(db: MySqlPool) -> (AppState, SessionRepository) {
        let db_conn = Database { pool: db };
        let placeholder_client = BasicClient::new(
            ClientId::new("test-client-id".to_string()),
            Some(ClientSecret::new("test-client-secret".to_string())),
            AuthUrl::new("https://test.auth.url".to_string()).unwrap(),
            Some(TokenUrl::new("https://test.token.url".to_string()).unwrap())
        );
        let app_state = AppState::new(db_conn, placeholder_client).await.unwrap();
        let session_repository = SessionRepository::new(&app_state.database);
        (app_state, session_repository)
    }

    fn build_cookies(cookie_name: &str, session_id: &str) -> Cookie {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::COOKIE,
            format!("{}={}; SameSite=Lax; HttpOnly; Secure; Path=/", cookie_name, session_id)
                .parse()
                .unwrap(),
        );
        headers.typed_get::<Cookie>().unwrap()
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_validate_csrf_token_with_valid_session(db: MySqlPool) {
        let (app_state, _) = setup(db).await;

        let auth_request = super::AuthRequest {
            code: "test_code".to_string(),
            state: "test_csrf_token".to_string(),
        };        

        let cookies = build_cookies("SESSION", "test_session_id");
        let result = validate_csrf_token(&app_state, &auth_request, &cookies).await;

        let session = sqlx::query!(
            r#"SELECT expires_at FROM sessions WHERE session_id = ?"#,
            "test_session_id"
        )
        .fetch_one(app_state.database.get_pool())
        .await
        .expect("Failed to fetch session");

        assert!(result.is_ok());
        assert!(session.expires_at.unwrap() < Utc::now());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_validate_csrf_token_expired_session(db: MySqlPool) {
        let (app_state, _) = setup(db).await;

        let auth_request = super::AuthRequest {
            code: "test_code".to_string(),
            state: "expired_session_id".to_string(),
        };

        let cookies = build_cookies("SESSION", "expired_session_id");
        let response = validate_csrf_token(&app_state, &auth_request, &cookies).await;

        assert_error!(response, &AppError::DatabaseError(String::new()));
    }

    #[sqlx::test]
    async fn test_validate_csrf_token_missing_session_cookie(db: MySqlPool) {
        let (app_state, _) = setup(db).await;

        let auth_request = super::AuthRequest {
            code: "test_code".to_string(),
            state: "test_session_id".to_string(),
        };

        let cookies = build_cookies("NoSessionIdCookie", "123");
        let response = validate_csrf_token(&app_state, &auth_request, &cookies).await;

        assert_error!(response, &AppError::InternalServerError(String::new()));
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_validate_csrf_token_invalid_csrf_token(db: MySqlPool) {
        let (app_state, _) = setup(db).await;

        let auth_request = super::AuthRequest {
            code: "invalid_code".to_string(),
            state: "test_session_id".to_string(),
        };

        let cookies = build_cookies("SESSION", "test_session_id");
        let response = validate_csrf_token(&app_state, &auth_request, &cookies).await;

        assert_error!(response, &AppError::TokenError(
            TokenError::GenericTokenError(String::new())
        ));
    }
}
