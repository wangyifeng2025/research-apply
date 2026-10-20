use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind: String,
    pub cors_origin: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: required_env("DATABASE_URL")?,
            bind: std::env::var("SERVER_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_owned()),
            cors_origin: std::env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_owned()),
        })
    }
}

fn required_env(name: &'static str) -> Result<String, AppError> {
    std::env::var(name).map_err(|_| AppError::MissingEnv(name))
}
