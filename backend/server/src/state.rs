use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::config::Config;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

impl AppState {
    pub async fn connect(config: &Config) -> Result<Self, AppError> {
        let mut options = ConnectOptions::new(config.database_url.clone());
        options
            .max_connections(10)
            .min_connections(1)
            .sqlx_logging(true);
        let db = Database::connect(options).await?;
        Ok(Self { db })
    }
}
