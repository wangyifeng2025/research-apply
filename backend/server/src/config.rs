use crate::error::AppError;

pub const AUTH_COOKIE: &str = "research_apply_session";
pub const SESSION_HOURS: i64 = 8;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub bind: String,
    pub cors_origin: String,
    pub jwt_secret: String,
    pub cookie_secure: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            database_url: required_env("DATABASE_URL")?,
            bind: std::env::var("SERVER_BIND").unwrap_or_else(|_| "127.0.0.1:8080".to_owned()),
            cors_origin: std::env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_owned()),
            jwt_secret: required_env("JWT_SECRET")?,
            cookie_secure: std::env::var("COOKIE_SECURE")
                .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("true")),
        })
    }

    pub fn database_only() -> Result<String, AppError> {
        dotenvy::dotenv().ok();
        required_env("DATABASE_URL")
    }
}

fn required_env(name: &'static str) -> Result<String, AppError> {
    std::env::var(name).map_err(|_| AppError::MissingEnv(name))
}
