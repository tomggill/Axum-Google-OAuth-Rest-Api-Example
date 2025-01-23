pub mod config;
pub mod handler;
pub mod middleware;
pub mod route;
pub mod service;
pub mod state;
pub mod repository;
pub mod error;

#[cfg(test)]
pub mod test_utils;


use anyhow::{Context, Result};
use axum::{extract::State, response::IntoResponse};
use config::{database::Database, parameter};
use error::app_error::AppError;
use http::Method;
use middleware::log;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl};
use route::create_router;
use serde::{Deserialize, Serialize};
use state::app_state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    parameter::init();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("sqlx=debug,{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting up the application...");


    let database_url = parameter::get("DATABASE_URL")?;
    let db = Database::new(&database_url).await?;
    let oauth_client = get_oauth_client()?;
    let app_state = AppState::new(db, oauth_client).await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    let app = create_router(app_state)
        .await
        .layer(cors)
        .layer(axum::middleware::from_fn(log::log_request));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .context("failed to bind TcpListener")
        .unwrap();

    tracing::debug!(
        "Application started - Listening on {}",
        listener
            .local_addr()
            .context("failed to return local address")
            .unwrap()
    );

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// TODO - Bad naming - need to redo the structs for google responses.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    sub: String,
    given_name: String,
    family_name: String,
    email: String,
}

async fn index(State(app_state): State<AppState>) -> impl IntoResponse {
    match app_state.user_context.read().await.as_ref() {
        Some(user) => format!(
            "Hey {}! You're logged in!\nYou may now access `/protected`.\nLog out with `/logout`.",
            user.name
        ),
        None => "You're not logged in.\nVisit `/auth/google` to do so.".to_string(),
    }
}

async fn protected(State(app_state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    match app_state.user_context.read().await.as_ref() {
        Some(user) => Ok(format!("Welcome to the protected area, {}!", user.name)),
        None => Err(anyhow::anyhow!("You're not logged in.").into()),
    }
}

fn get_oauth_client() -> Result<BasicClient, AppError> {
    let client_id = parameter::get("GOOGLE_CLIENT_ID")?;
    let client_secret = parameter::get("GOOGLE_CLIENT_SECRET")?;
    let redirect_url = parameter::get("GOOGLE_REDIRECT_URI")?;
    let auth_url = parameter::get("GOOGLE_AUTH_URI")?;
    let token_url = parameter::get("GOOGLE_TOKEN_URI")?;
    let revocation_url = parameter::get("GOOGLE_REVOCATION_URI")?;


    Ok(BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(auth_url).context("failed to create new authorization server URL")?,
            Some(TokenUrl::new(token_url).context("failed to create new token endpoint URL")?),
        )
        .set_redirect_uri(
            RedirectUrl::new(redirect_url).context("failed to create new redirection URL")?,
        )
        .set_revocation_uri(
            RevocationUrl::new(revocation_url).context("failed to create new revocation URL")?,
        )
    )
}
