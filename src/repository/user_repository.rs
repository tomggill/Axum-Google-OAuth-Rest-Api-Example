use std::sync::Arc;

use anyhow::Context;
use async_trait::async_trait;

use crate::{config::database::{Database, DatabaseTrait}, error::app_error::AppError, state::app_state::UserContext};


#[derive(Clone)]
pub struct UserRepository {
    pub(crate) db_conn: Arc<Database>,
}

#[async_trait]
pub trait UserRepositoryTrait {
    fn new(db_conn: &Arc<Database>) -> Self;
    async fn add_user(&self, google_id: &str, email: &str, first_name: &str, last_name: &str) -> Result<u64, AppError>;
    async fn find_user_by_google_id(&self, google_id: &str) -> Result<Option<UserContext>, AppError>;
}

#[async_trait]
impl UserRepositoryTrait for UserRepository {
    fn new(db_conn: &Arc<Database>) -> Self {
        Self {
            db_conn: Arc::clone(db_conn),
        }
    }

    async fn add_user(&self, google_id: &str, email: &str, first_name: &str, last_name: &str) -> Result<u64, AppError> {
        tracing::debug!("Creating a new user");
        let user = sqlx::query!(
            r#"
                INSERT INTO users (google_id, email, first_name, last_name)
                VALUES (?, ?, ?, ?)
            "#,
            google_id,
            email,
            first_name,
            last_name,
        )
        .execute(self.db_conn.get_pool())
        .await
        .context("Failed to insert user into database")?;

        Ok(user.last_insert_id())
    }

    async fn find_user_by_google_id(&self, google_id: &str) -> Result<Option<UserContext>, AppError> {
        let user_context = sqlx::query_as!(
            UserContext,
            r#"
            SELECT 
                CAST(id as unsigned) AS user_id, 
                email, 
                first_name AS name
            FROM users
            WHERE google_id = ?
            "#,
            google_id
        )
        .fetch_optional(self.db_conn.get_pool())
        .await?;
    
        Ok(user_context)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::MySqlPool;

    use crate::state::app_state::AppState;
    use crate::config::database::Database;

    use super::{UserRepository, UserRepositoryTrait};

    async fn setup(db: MySqlPool) -> (AppState, UserRepository) {
        let db_conn = Database { pool: db };
        let app = AppState::new(db_conn).await.unwrap();
        let user_repository = UserRepository::new(&app.database);
        (app, user_repository)
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/users.sql"))]
    async fn test_create_valid_user(db: MySqlPool) {
        let (_, user_repository) = setup(db).await;

        let response = user_repository.add_user("123456789", "test@lift.com", "John", "Smith").await;
        assert!(response.is_ok());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/users.sql"))]
    async fn test_create_duplicate_user(db: MySqlPool) {
        let (_, user_repository) = setup(db).await;

        let _ = user_repository.add_user("123456789", "test@lift.com", "John", "Smith").await;
        let response = user_repository.add_user("123456789", "test@lift.com", "John", "Smith").await;
        assert!(response.is_err());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/users.sql"))]
    async fn test_find_non_existent_user(db: MySqlPool) {
        let (_, user_repository) = setup(db).await;

        let user_context = user_repository.find_user_by_google_id("non_existent_id").await.unwrap();
        assert!(user_context.is_none());
    }

    #[sqlx::test(fixtures("./../../tests/fixtures/users.sql"))]
    async fn test_find_existing_user(db: MySqlPool) {
        let (_, user_repository) = setup(db).await;

        let _ = user_repository.add_user("123456789", "test@lift.com", "John", "Smith").await;
        let user_context = user_repository.find_user_by_google_id("123456789").await.unwrap();
        assert!(user_context.is_some());
    }
}
