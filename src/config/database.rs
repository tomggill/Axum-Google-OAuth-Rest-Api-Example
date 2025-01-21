use anyhow::Context;
use async_trait::async_trait;
use sqlx::{mysql::MySqlPoolOptions, MySqlPool};

use crate::error::app_error::AppError;

use super::parameter;

#[derive(Clone)]
pub struct Database {
    pub pool: MySqlPool,
}

#[async_trait]
pub trait DatabaseTrait {
    async fn new() -> Result<Self, AppError>
    where
        Self: Sized;
    fn get_pool(&self) -> &MySqlPool;
}

#[async_trait]
impl DatabaseTrait for Database {
    async fn new() -> Result<Self, AppError> {
        let database_url = parameter::get("DATABASE_URL")?;
        let pool = MySqlPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
            .context("Failed to connect to the database")?;
        tracing::info!("Connection to the database is successful!");
        Ok(Self { pool })
    }

    fn get_pool(&self) -> &MySqlPool {
        &self.pool
    }
}
