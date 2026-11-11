use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::cycles::{self, CreateCycleInput, CycleView, UpdateCycleInput};
use crate::state::AppState;

pub async fn list(
    State(state): State<AppState>,
    actor: CurrentUser,
) -> Result<Json<Value>, AppError> {
    let items = cycles::list(&state.db, &actor).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn current(State(state): State<AppState>) -> Result<Json<Option<CycleView>>, AppError> {
    Ok(Json(cycles::current(&state.db).await?))
}

pub async fn create(
    State(state): State<AppState>,
    actor: CurrentUser,
    Json(input): Json<CreateCycleInput>,
) -> Result<(StatusCode, Json<CycleView>), AppError> {
    let cycle = cycles::create(&state, &actor, input).await?;
    Ok((StatusCode::CREATED, Json(cycle)))
}

pub async fn update(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCycleInput>,
) -> Result<Json<CycleView>, AppError> {
    Ok(Json(cycles::update(&state, &actor, id, input).await?))
}
