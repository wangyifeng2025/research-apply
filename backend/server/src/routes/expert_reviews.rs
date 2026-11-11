use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::expert_reviews::{self, ExpertDetail, ExpertReviewInput, SubmittedReview};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct QueueQuery {
    scope: Option<String>,
}

pub async fn queue(
    State(state): State<AppState>,
    actor: CurrentUser,
    Query(query): Query<QueueQuery>,
) -> Result<Json<Value>, AppError> {
    let scope = query.scope.as_deref().unwrap_or("pending");
    let items = expert_reviews::queue(&state.db, &actor, scope).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn detail(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<ExpertDetail>, AppError> {
    Ok(Json(expert_reviews::open(&state.db, &actor, id).await?))
}

pub async fn submit(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ExpertReviewInput>,
) -> Result<Json<SubmittedReview>, AppError> {
    Ok(Json(
        expert_reviews::submit(&state.db, &actor, id, input).await?,
    ))
}
