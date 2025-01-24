use anyhow::Context;
use oauth2::{basic::BasicClient, reqwest::async_http_client, AccessToken, AuthorizationCode, CsrfToken, RefreshToken, Scope, StandardRevocableToken, TokenResponse};
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};

use crate::{config:: parameter, error::{app_error::AppError, token_error::TokenError}, User};

#[derive(Deserialize, Serialize, Debug)]
pub struct GoogleTokenInfo {
    pub audience: String,  // The audience for which the token was issued
    pub email: String,     // The email associated with the access token
    pub expires_in: i64,   // Expiration time in seconds
    pub issued_to: String, // The client_id the token was issued to
    pub user_id: String,   // The user's unique ID
}

#[derive(Clone)]
pub struct GoogleTokenService {
    oauth_client: BasicClient,
    http_client: Client,
}

pub trait TokenServiceTrait {
    fn new(oauth_client: BasicClient) -> Self;
    async fn generate_authorisation_url(&self) -> Result<(Url, CsrfToken), AppError>;
    async fn exchange_authorisation_code(&self, code: String) -> Result<(AccessToken, RefreshToken), AppError>;
    async fn refresh_access_token(&self, refresh_token: String) -> Result<AccessToken, AppError>;
    async fn revoke_token(&self, token: String) -> Result<(), AppError>;
    async fn get_token_info(&self, access_token: &str) -> Result<GoogleTokenInfo, AppError>;
    async fn get_user_info(&self, access_token: &str) -> Result<User, AppError>;
}

impl TokenServiceTrait for GoogleTokenService {
    fn new(oauth_client: BasicClient) -> Self {
        Self {
            oauth_client,
            http_client: Client::new(),
        }
    }

    async fn generate_authorisation_url(&self) -> Result<(Url, CsrfToken), AppError> {
        let (auth_url, csrf_token) = self.oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new(
                parameter::get("GOOGLE_EMAIL_SCOPE")?,
            ))
            .add_scope(Scope::new(
                parameter::get("GOOGLE_PROFILE_SCOPE")?,
            ))
            .add_extra_param("access_type", "offline")
            .add_extra_param("prompt", "consent")
            .url();

        Ok((auth_url, csrf_token))
    }

    async fn exchange_authorisation_code(&self, code: String) -> Result<(AccessToken, RefreshToken), AppError> {
        let token = self.oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .request_async(async_http_client)
            .await
            .context("failed in sending request to authorization server")?;

        let access_token = token.access_token().to_owned();

        let refresh_token = token.refresh_token().context("Missing refresh token")?.to_owned();
    
        Ok((access_token, refresh_token))
    }

    async fn refresh_access_token(
        &self,
        refresh_token: String,
    ) -> Result<AccessToken, AppError> {
        let token_response = self.oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .context("failed to refresh access token")?;
    
        Ok(token_response.access_token().to_owned())
    }

    async fn revoke_token(&self, token: String) -> Result<(), AppError> {
        let token = AccessToken::new(token);
        let revocable_token: StandardRevocableToken = token.into();

        let response = self.oauth_client.revoke_token(revocable_token)?.request_async(async_http_client).await;

        if let Err(error) = response {
            return Err(TokenError::InvalidToken)?;
        } 
        Ok(())
    }

    async fn get_token_info(
        &self,
        access_token: &str,
    ) -> Result<GoogleTokenInfo, AppError> {
        let token_info_url = parameter::get("GOOGLE_TOKEN_INFO_URI")?;
        let response = self
            .http_client
            .get(token_info_url)
            .bearer_auth(access_token)
            .send()
            .await
            .context("Failed to send request to Google's token info endpoint")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Google token validation failed").into());
        }

        let google_token_info = response
            .json::<GoogleTokenInfo>()
            .await
            .context("Failed to parse token info from response")?;

        Ok(google_token_info)
    }

    async fn get_user_info(&self, access_token: &str) -> Result<User, AppError> {
        let user_data: User = self.http_client
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .context("failed in sending request to target Url")?
            .json::<User>()
            .await
            .context("failed to deserialize response as JSON")?;

        Ok(user_data)
    }
}
