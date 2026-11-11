use axum::Json;
use axum::extract::State;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::{profile, profile::ProfileView};
use crate::state::AppState;

pub async fn show(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<ProfileView>, AppError> {
    Ok(Json(profile::show(&state, &user).await?))
}

pub async fn update(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<profile::ProfileInput>,
) -> Result<Json<ProfileView>, AppError> {
    Ok(Json(profile::update(&state, &user, body).await?))
}
