use std::sync::Arc;

use axum::extract::FromRef;
use reqwest::Client;
use tokio::sync::RwLock;

use crate::{config::database::Database, error::app_error::AppError, repository::{session_repository::{SessionRepository, SessionRepositoryTrait}, user_repository::{UserRepository, UserRepositoryTrait}}, service::{google_token_service::{GoogleTokenService, TokenServiceTrait}, user_service::UserService}};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct UserContext {
    pub user_id: u64,
    pub email: String,
    pub name: String,
}

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub http_client: Client,
    pub user_context: Arc<RwLock<Option<UserContext>>>,
    pub google_token_service: GoogleTokenService,
    pub user_service: UserService,
    pub user_repository: UserRepository,
    pub session_repository: SessionRepository,
}

impl FromRef<AppState> for GoogleTokenService {
    fn from_ref(state: &AppState) -> Self {
        state.google_token_service.clone()
    }
}

impl AppState {
    pub async fn new(db: Database) -> Result<Self, AppError> {
        let db_conn = Arc::new(db);
        Ok(Self {
            database: db_conn.clone(),
            http_client: Client::new(),
            user_context: Arc::new(RwLock::new(None)),
            google_token_service: GoogleTokenService::new(),
            user_service: UserService::new(&db_conn),
            user_repository: UserRepository::new(&db_conn),
            session_repository: SessionRepository::new(&db_conn),
        })
    }

    pub async fn get_user_id(&self) -> Option<u64> {
        self.user_context.read().await.as_ref().map(|user| user.user_id)
    }

    pub async fn set_user_context(&self, user_context: UserContext) {
        let mut user_context_lock = self.user_context.write().await;
        *user_context_lock = Some(user_context);
    }

    pub async fn clear_user_context(&self) {
        let mut user_context_lock = self.user_context.write().await;
        *user_context_lock = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::database::Database;
    use sqlx::MySqlPool;
    
    async fn setup(db: MySqlPool) -> AppState {
        let db_conn = Database { pool: db };
        AppState::new(db_conn).await.unwrap()
    }

    #[sqlx::test]
    async fn test_user_context_operations(db: MySqlPool) {
        let app_state = setup(db).await;
        let test_user = UserContext { user_id: 1, email: "test@lift.com".to_string(), name: "John".to_string() };

        assert!(app_state.get_user_id().await.is_none());

        app_state.set_user_context(test_user).await;
        assert_eq!(app_state.get_user_id().await, Some(1));

        app_state.clear_user_context().await;
        assert!(app_state.get_user_id().await.is_none());
    }

    #[sqlx::test]
    async fn test_concurrent_access(db: MySqlPool) {
        let app_state = setup(db).await;
        let app_state_clone = app_state.clone();
        let app_state_clone2 = app_state.clone();
        let test_user = UserContext { user_id: 1, email: "test@lift.com".to_string(), name: "John".to_string() };

        let handle1 = tokio::spawn(async move {
            app_state.set_user_context(test_user).await;
        });

        let handle2 = tokio::spawn(async move {
            app_state_clone.clear_user_context().await;
        });

        tokio::try_join!(handle1, handle2).unwrap();

        let final_state = app_state_clone2.get_user_id().await;

        assert!(final_state.is_none() || final_state == Some(1));
    }
}
