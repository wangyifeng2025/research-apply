use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::applications;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FamilyQuery {
    family: String,
}

#[derive(Deserialize)]
pub struct SubmitBody {
    commitment_accepted: bool,
}

pub async fn create(
    State(state): State<AppState>,
    user: CurrentUser,
    Json(body): Json<applications::CreateInput>,
) -> Result<(StatusCode, Json<applications::ApplicationView>), AppError> {
    let view = applications::create(&state, &user, body).await?;
    Ok((StatusCode::CREATED, Json(view)))
}

pub async fn mine(
    State(state): State<AppState>,
    user: CurrentUser,
) -> Result<Json<Value>, AppError> {
    let items = applications::mine(&state.db, &user).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn detail(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<applications::ApplicationView>, AppError> {
    Ok(Json(applications::detail(&state.db, &user, id).await?))
}

pub async fn destroy(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    applications::delete_draft(&state, &user, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn save_section(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, section)): Path<(Uuid, String)>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, AppError> {
    applications::save_section(&state, &user, id, &section, body).await?;
    Ok(Json(json!({ "saved": section })))
}

pub async fn submit(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(body): Json<SubmitBody>,
) -> Result<Json<applications::ApplicationView>, AppError> {
    Ok(Json(applications::submit(&state, &user, id, body.commitment_accepted).await?))
}

pub async fn withdraw(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<applications::ApplicationView>, AppError> {
    Ok(Json(applications::withdraw(&state, &user, id).await?))
}

pub async fn subjects(
    State(state): State<AppState>,
    Query(query): Query<FamilyQuery>,
) -> Result<Json<Value>, AppError> {
    let subjects = applications::subjects(&state.db, &query.family).await?;
    Ok(Json(json!({ "subjects": subjects })))
}
