mod health;
mod meta;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::get;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::config::Config;
use crate::error::AppError;
use crate::state::AppState;

pub fn router(state: AppState, config: &Config) -> Result<Router, AppError> {
    let origin = HeaderValue::from_str(&config.cors_origin)
        .map_err(|_| AppError::InvalidCorsOrigin(config.cors_origin.clone()))?;
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE]);

    Ok(Router::new()
        .route("/health", get(health::health))
        .route("/api/meta", get(meta::meta))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http()))
}
