use std::sync::Arc;

use async_trait::async_trait;
use chrono::{Duration, Utc};

use crate::{config::database::{Database, DatabaseTrait}, error::app_error::AppError};


#[derive(Clone)]
pub struct SessionRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait SessionRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn add_csrf_token(&self, session_id: &str, csrf_token: &str) -> Result<(), AppError>;
    async fn expire_session(&self, session_id: &str) -> Result<(), AppError>;
    async fn get_csrf_token_by_session_id(&self, session_id: &str) -> Result<String, AppError>;
}

#[async_trait]
impl SessionRepositoryTrait for SessionRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn add_csrf_token(&self, session_id: &str, csrf_token: &str) -> Result<(), AppError> {
        let expires_at = Utc::now() + Duration::hours(1);
        sqlx::query!(
            r#"
                INSERT INTO sessions (session_id, csrf_token, expires_at)
                VALUES (?, ?, ?)
                "#,
            session_id,
            csrf_token,
            expires_at
        )
        .execute(self.db_conn.get_pool())
        .await?;
    
        Ok(())
    }
    
    async fn expire_session(&self, session_id: &str) -> Result<(), AppError> {
        sqlx::query!(
            r#"
                UPDATE sessions
                SET expires_at = NOW()
                WHERE session_id = ?
            "#,
            session_id
        )
        .execute(self.db_conn.get_pool())
        .await?;
    
        Ok(())
    }

    async fn get_csrf_token_by_session_id (&self, session_id: &str) -> Result<String, AppError> {
        let session = sqlx::query!(
            r#"
                SELECT csrf_token FROM sessions WHERE session_id = ? AND expires_at > NOW()
            "#,
            session_id
        )
        .fetch_one(self.db_conn.get_pool())
        .await?;
    
        Ok(session.csrf_token)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use ::chrono::{DateTime, Utc};
    use sqlx::MySqlPool;

    use crate::repository::session_repository::SessionRepositoryTrait;
    use crate::config::database::Database;

    use super::SessionRepository;

    async fn get_session_repository(db: MySqlPool) -> SessionRepository {
        let db_conn = Database { pool: db };
        SessionRepository::new(&Arc::new(db_conn))
    }

    #[sqlx::test]
    async fn test_add_csrf_token(db: MySqlPool) {
        let session_repository = get_session_repository(db).await;

        let response = session_repository.add_csrf_token("8M2q73XaSqa67eE8Zi", "eQ5MCnz-erkK9Xfm4O3JRA").await;
        assert!(response.is_ok());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_get_csrf_token_by_session_id(db: MySqlPool) {
        let session_repository = get_session_repository(db).await;

        let csrf_token = session_repository.get_csrf_token_by_session_id("test_session_id").await;
        assert!(csrf_token.is_ok());
        assert_eq!(csrf_token.unwrap(), "test_csrf_token");
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_get_csrf_token_by_expired_session_id(db: MySqlPool) {
        let session_repository = get_session_repository(db).await;

        let result = session_repository.get_csrf_token_by_session_id("expired_session_id").await;
        assert!(result.is_err());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_get_csrf_token_by_non_existent_session_id(db: MySqlPool) {
        let session_repository = get_session_repository(db).await;

        let result = session_repository.get_csrf_token_by_session_id("non_existent_session_id").await;
        assert!(result.is_err());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/session.sql"))]
    async fn test_expire_session(db: MySqlPool) {
        let session_repository = get_session_repository(db.clone()).await;

        let result = session_repository.expire_session("test_session_id").await;
        assert!(result.is_ok());

        let session = sqlx::query!(
            r#"
            SELECT expires_at FROM sessions WHERE session_id = ?
            "#,
            "test_session_id"
        )
        .fetch_one(&db)
        .await
        .expect("Failed to fetch session");

        let current_time: DateTime<Utc> = Utc::now();

        assert!(session.expires_at.unwrap() < current_time);
    }
}
