use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde_json::{Value, json};

use crate::error::AppError;
use crate::state::AppState;

pub async fn health(State(state): State<AppState>) -> Result<(StatusCode, Json<Value>), AppError> {
    state.db.ping().await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "ok",
            "database": "up"
        })),
    ))
}
