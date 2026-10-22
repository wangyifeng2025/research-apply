mod applications;
mod auth;
mod health;
mod meta;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{get, post, put};
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
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
        .allow_credentials(true);

    Ok(Router::new()
        .route("/health", get(health::health))
        .route("/api/meta", get(meta::meta))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/budget-subjects", get(applications::subjects))
        .route("/api/applications", post(applications::create))
        .route("/api/applications/mine", get(applications::mine))
        .route("/api/applications/{id}", get(applications::detail).delete(applications::destroy))
        .route("/api/applications/{id}/{section}", put(applications::save_section))
        .route("/api/applications/{id}/submit", post(applications::submit))
        .route("/api/applications/{id}/withdraw", post(applications::withdraw))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http()))
}
