use anyhow::Context;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

use crate::error::app_error::AppError;

#[derive(Clone)]
pub struct Database {
    pub pool: MySqlPool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, AppError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .context("Failed to connect to the database")?;
        tracing::info!("Connection to the database is successful!");
        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &MySqlPool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use sqlx::MySqlPool;

    use crate::config::database::Database;

    #[tokio::test]
    async fn test_database_new_failure() {
        let result = Database::new("invalid_url").await;
        assert!(result.is_err());
    }

    #[sqlx::test]
    async fn test_database_pool_access(db: MySqlPool) {
        let database = Database { pool: db };
        assert!(database.get_pool().acquire().await.is_ok());
    }

    #[sqlx::test]
    async fn test_database_connection(db: MySqlPool) {
        let database = Database { pool: db };
        let result = sqlx::query("SELECT 1").execute(database.get_pool()).await;
        assert!(result.is_ok());
    }
}
