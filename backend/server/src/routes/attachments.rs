use axum::Json;
use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::{attachments, attachments::content_disposition};
use crate::state::AppState;
use crate::support::blobs::REQUEST_LIMIT;

pub fn body_limit() -> DefaultBodyLimit {
    DefaultBodyLimit::max(REQUEST_LIMIT)
}

pub async fn upload(
    State(state): State<AppState>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<attachments::AttachmentView>), AppError> {
    let view = attachments::upload(&state, &user, id, multipart).await?;
    Ok((StatusCode::CREATED, Json(view)))
}

pub async fn download(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, AppError> {
    let file = attachments::download(&state, &user, id, attachment_id).await?;
    let content_type = HeaderValue::from_str(&file.content_type)
        .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));
    let disposition = HeaderValue::from_str(&content_disposition(&file.file_name))
        .unwrap_or_else(|_| HeaderValue::from_static("attachment"));
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CONTENT_LENGTH, file.bytes.len())
        .body(Body::from(file.bytes))
        .map_err(|_| AppError::Internal("附件响应生成失败".to_owned()))
}

pub async fn remove(
    State(state): State<AppState>,
    user: CurrentUser,
    Path((id, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    attachments::delete(&state, &user, id, attachment_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
