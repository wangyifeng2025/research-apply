mod admin;
mod applications;
mod attachments;
mod auth;
mod cycles;
mod department_reviews;
mod expert_reviews;
mod health;
mod meta;
mod office_reviews;
mod profile;
mod stats;

use axum::Router;
use axum::http::{HeaderValue, Method, header};
use axum::routing::{delete, get, post, put};
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
        .route("/api/profile", get(profile::show).patch(profile::update))
        .route("/api/stats/overview", get(stats::overview))
        .route("/api/stats/export", get(stats::export))
        .route("/api/budget-subjects", get(applications::subjects))
        .route("/api/departments", get(admin::departments))
        .route("/api/cycles/current", get(cycles::current))
        .route("/api/cycles", get(cycles::list).post(cycles::create))
        .route("/api/cycles/{id}", axum::routing::patch(cycles::update))
        .route("/api/admin/departments", post(admin::create_department))
        .route(
            "/api/admin/departments/{id}",
            axum::routing::patch(admin::update_department).delete(admin::delete_department),
        )
        .route(
            "/api/admin/users",
            get(admin::users).post(admin::create_user),
        )
        .route(
            "/api/admin/users/{id}",
            axum::routing::patch(admin::update_user).delete(admin::delete_user),
        )
        .route(
            "/api/admin/users/{id}/reset-password",
            post(admin::reset_password),
        )
        .route("/api/applications", post(applications::create))
        .route("/api/applications/mine", get(applications::mine))
        .route(
            "/api/applications/{id}",
            get(applications::detail).delete(applications::destroy),
        )
        .route(
            "/api/applications/{id}/form.pdf",
            get(applications::export_pdf),
        )
        .route(
            "/api/applications/{id}/{section}",
            put(applications::save_section),
        )
        .route(
            "/api/applications/{id}/attachments",
            post(attachments::upload).layer(attachments::body_limit()),
        )
        .route(
            "/api/applications/{id}/attachments/{attachment_id}",
            get(attachments::download).delete(attachments::remove),
        )
        .route("/api/applications/{id}/submit", post(applications::submit))
        .route(
            "/api/applications/{id}/withdraw",
            post(applications::withdraw),
        )
        .route(
            "/api/applications/{id}/timeline",
            get(applications::timeline),
        )
        .route("/api/department-reviews", get(department_reviews::queue))
        .route(
            "/api/department-reviews/{id}",
            get(department_reviews::detail).post(department_reviews::decide),
        )
        .route("/api/office-reviews", get(office_reviews::queue))
        .route("/api/office-reviews/experts", get(office_reviews::experts))
        .route(
            "/api/office-reviews/{id}",
            get(office_reviews::detail).post(office_reviews::decide),
        )
        .route(
            "/api/office-reviews/{id}/assignments",
            post(office_reviews::assign),
        )
        .route(
            "/api/office-reviews/{id}/assignments/{assignment_id}",
            delete(office_reviews::revoke),
        )
        .route(
            "/api/office-reviews/{id}/conclusion",
            post(office_reviews::conclude),
        )
        .route("/api/expert-reviews", get(expert_reviews::queue))
        .route(
            "/api/expert-reviews/{id}",
            get(expert_reviews::detail).post(expert_reviews::submit),
        )
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http()))
}
