use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::{application_form, applications, timeline};
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

pub async fn export_pdf(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Response, AppError> {
    let (name, bytes) = application_form::export(&state, &user, id).await?;
    let disposition = HeaderValue::from_str(&content_disposition(&name))
        .map_err(|_| AppError::Internal("导出文件名无效".to_owned()))?;
    let content_type = HeaderValue::from_static("application/pdf");
    Ok((
        [
            (header::CONTENT_TYPE, content_type),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    )
        .into_response())
}

fn content_disposition(name: &str) -> String {
    format!(
        "attachment; filename=\"application.pdf\"; filename*=UTF-8''{}",
        percent_encode(name)
    )
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
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
    Ok(Json(
        applications::submit(&state, &user, id, body.commitment_accepted).await?,
    ))
}

pub async fn timeline(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let events = timeline::list(&state.db, &user, id).await?;
    Ok(Json(json!({ "events": events })))
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
