use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::office_reviews::{
    self, AssignInput, ConclusionInput, OfficeDecisionInput, OfficeDetail, OfficeListItem,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct QueueQuery {
    scope: Option<String>,
    q: Option<String>,
}

pub async fn queue(
    State(state): State<AppState>,
    actor: CurrentUser,
    Query(query): Query<QueueQuery>,
) -> Result<Json<Value>, AppError> {
    let scope = query.scope.as_deref().unwrap_or("pending");
    let items = office_reviews::queue(&state.db, &actor, scope).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn experts(
    State(state): State<AppState>,
    actor: CurrentUser,
    Query(query): Query<QueueQuery>,
) -> Result<Json<Value>, AppError> {
    let items = office_reviews::experts(&state.db, &actor, query.q.as_deref()).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn detail(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<OfficeDetail>, AppError> {
    Ok(Json(office_reviews::open(&state.db, &actor, id).await?))
}

pub async fn decide(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<OfficeDecisionInput>,
) -> Result<Json<OfficeListItem>, AppError> {
    Ok(Json(
        office_reviews::decide(&state.db, &actor, id, input).await?,
    ))
}

pub async fn assign(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<AssignInput>,
) -> Result<Json<OfficeDetail>, AppError> {
    Ok(Json(
        office_reviews::assign(&state.db, &actor, id, input).await?,
    ))
}

pub async fn revoke(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path((id, assignment_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<OfficeDetail>, AppError> {
    Ok(Json(
        office_reviews::revoke(&state.db, &actor, id, assignment_id).await?,
    ))
}

pub async fn conclude(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ConclusionInput>,
) -> Result<Json<OfficeListItem>, AppError> {
    Ok(Json(
        office_reviews::conclude(&state.db, &actor, id, input).await?,
    ))
}
