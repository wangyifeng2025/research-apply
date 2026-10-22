use sea_orm::{ConnectOptions, Database, DatabaseConnection};

use crate::config::Config;
use crate::error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
}

impl AppState {
    pub async fn connect(config: &Config) -> Result<Self, AppError> {
        Ok(Self {
            db: connect_database(&config.database_url).await?,
            config: config.clone(),
        })
    }
}

pub async fn connect_database(database_url: &str) -> Result<DatabaseConnection, AppError> {
    let mut options = ConnectOptions::new(database_url.to_owned());
    options
        .max_connections(10)
        .min_connections(1)
        .sqlx_logging(true);
    Ok(Database::connect(options).await?)
}
