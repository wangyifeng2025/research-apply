//! 申报附件：上传、下载、删除。
//!
//! 只有负责人可以在草稿、系部退回、科研处退回时增删。下载按角色鉴权：
//! 负责人、本部门系审核人、科研处和管理员可以看到全部附件；
//! 已指派的专家看不到签字件，也不能凭编号下载。

use axum::extract::Multipart;
use entity::application_attachments;
use entity::applications;
use entity::expert_assignments;
use entity::sea_orm_active_enums::{ApplicationStatus, AttachmentKind, UserRole};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::blobs::{LocalDirectory, exceeds_limit, safe_file_name, sniff};
use crate::support::time::now;

#[derive(Debug, Serialize)]
pub struct AttachmentView {
    pub id: Uuid,
    pub kind: String,
    pub file_name: String,
    pub content_type: String,
    pub byte_size: i64,
    pub created_at: String,
}

pub struct DownloadedFile {
    pub file_name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

pub async fn list_views(
    db: &DatabaseConnection,
    application_id: Uuid,
    include_signature: bool,
) -> Result<Vec<AttachmentView>, AppError> {
    let rows = application_attachments::Entity::find()
        .filter(application_attachments::Column::ApplicationId.eq(application_id))
        .order_by_asc(application_attachments::Column::CreatedAt)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .filter(|row| visible_kind(&row.kind, include_signature))
        .map(to_view)
        .collect())
}

pub async fn upload(
    state: &AppState,
    user: &CurrentUser,
    application_id: Uuid,
    mut multipart: Multipart,
) -> Result<AttachmentView, AppError> {
    let app = load_application(&state.db, application_id).await?;
    ensure_owner_can_edit(user, &app)?;

    let mut kind_raw = String::new();
    let mut file_name = String::new();
    let mut bytes = Vec::new();
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("附件上传失败，请重新选择文件".to_owned()))?
    {
        let name = field.name().map(str::to_owned);
        match name.as_deref() {
            Some("kind") => {
                kind_raw = field
                    .text()
                    .await
                    .map_err(|_| AppError::BadRequest("附件上传失败，请重新选择文件".to_owned()))?;
            }
            Some("file") => {
                file_name = safe_file_name(field.file_name().unwrap_or("附件"));
                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|_| AppError::BadRequest("附件上传失败，请重新选择文件".to_owned()))?
                {
                    let next = bytes.len() as u64 + chunk.len() as u64;
                    if exceeds_limit(next) {
                        return Err(AppError::PayloadTooLarge(
                            "单个附件不能超过 20 MB".to_owned(),
                        ));
                    }
                    bytes.extend_from_slice(&chunk);
                }
            }
            _ => {}
        }
    }

    if bytes.is_empty() {
        return Err(invalid("file", "请选择要上传的文件"));
    }
    let detected = sniff(&bytes).ok_or_else(|| invalid("file", "只允许上传 PDF、JPG、PNG"))?;
    if !detected.matches_name(&file_name) {
        return Err(invalid("file", "文件扩展名与文件内容不一致"));
    }
    let kind = parse_kind(kind_raw.trim())?;
    let byte_size = i64::try_from(bytes.len())
        .map_err(|_| AppError::PayloadTooLarge("单个附件不能超过 20 MB".to_owned()))?;
    let key = format!("{}.{}", Uuid::new_v4(), detected.extension());
    let store = LocalDirectory::new(&state.config.attachment_dir);
    store.save(&key, &bytes).await?;

    let timestamp = now();
    let saved = application_attachments::ActiveModel {
        id: Set(Uuid::new_v4()),
        application_id: Set(application_id),
        kind: Set(kind),
        file_name: Set(file_name),
        content_type: Set(detected.content_type().to_owned()),
        byte_size: Set(byte_size),
        storage_key: Set(key.clone()),
        uploaded_by: Set(user.id),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    }
    .insert(&state.db)
    .await;
    let saved = match saved {
        Ok(row) => row,
        Err(error) => {
            if let Err(remove_error) = store.remove(&key).await {
                tracing::error!(error = %remove_error, "附件记录写入失败后删除文件失败");
            }
            return Err(error.into());
        }
    };
    record(
        &state.db,
        AuditEntry {
            actor_id: Some(user.id),
            action: "attachment_uploaded",
            target_type: "application",
            target_id: Some(application_id),
            detail: Some(json!({
                "attachment_id": saved.id,
                "kind": kind_code(&saved.kind),
                "byte_size": saved.byte_size,
            })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(to_view(saved))
}

pub async fn download(
    state: &AppState,
    user: &CurrentUser,
    application_id: Uuid,
    attachment_id: Uuid,
) -> Result<DownloadedFile, AppError> {
    let app = load_application(&state.db, application_id).await?;
    let scope = authorize_read(&state.db, user, &app).await?;
    let row = load_attachment(&state.db, application_id, attachment_id).await?;
    if scope == ReadScope::Blind && row.kind == AttachmentKind::Signature {
        return Err(AppError::NotFound("附件不存在".to_owned()));
    }
    let store = LocalDirectory::new(&state.config.attachment_dir);
    let bytes = store.load(&row.storage_key).await?;
    Ok(DownloadedFile {
        file_name: row.file_name,
        content_type: row.content_type,
        bytes,
    })
}

pub async fn delete(
    state: &AppState,
    user: &CurrentUser,
    application_id: Uuid,
    attachment_id: Uuid,
) -> Result<(), AppError> {
    let app = load_application(&state.db, application_id).await?;
    ensure_owner_can_edit(user, &app)?;
    let row = load_attachment(&state.db, application_id, attachment_id).await?;
    let result = application_attachments::Entity::delete_by_id(row.id)
        .exec(&state.db)
        .await?;
    if result.rows_affected == 0 {
        return Err(AppError::NotFound("附件不存在".to_owned()));
    }
    let store = LocalDirectory::new(&state.config.attachment_dir);
    if let Err(error) = store.remove(&row.storage_key).await {
        tracing::error!(error = %error, "附件记录已删除，文件删除失败");
    }
    record(
        &state.db,
        AuditEntry {
            actor_id: Some(user.id),
            action: "attachment_deleted",
            target_type: "application",
            target_id: Some(application_id),
            detail: Some(json!({
                "attachment_id": attachment_id,
                "kind": kind_code(&row.kind),
            })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    Ok(())
}

pub async fn storage_keys(
    db: &DatabaseConnection,
    application_id: Uuid,
) -> Result<Vec<String>, AppError> {
    let rows = application_attachments::Entity::find()
        .filter(application_attachments::Column::ApplicationId.eq(application_id))
        .all(db)
        .await?;
    Ok(rows.into_iter().map(|row| row.storage_key).collect())
}

pub async fn remove_stored(directory: &std::path::Path, keys: &[String]) {
    let store = LocalDirectory::new(directory);
    for key in keys {
        if let Err(error) = store.remove(key).await {
            tracing::error!(error = %error, "删除申报书时清理附件失败");
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ReadScope {
    Full,
    Blind,
}

async fn authorize_read(
    db: &DatabaseConnection,
    user: &CurrentUser,
    app: &applications::Model,
) -> Result<ReadScope, AppError> {
    if app.principal_id == user.id
        || user.has_role(UserRole::Admin)
        || user.has_role(UserRole::ResearchOffice)
    {
        return Ok(ReadScope::Full);
    }
    if user.has_role(UserRole::DepartmentReviewer) {
        return if user.department_id.is_some() && user.department_id == app.department_id {
            Ok(ReadScope::Full)
        } else {
            Err(AppError::Forbidden("只能查看本部门的申报附件".to_owned()))
        };
    }
    if user.has_role(UserRole::Expert) {
        let assigned = expert_assignments::Entity::find()
            .filter(expert_assignments::Column::ExpertId.eq(user.id))
            .filter(expert_assignments::Column::ApplicationId.eq(app.id))
            .one(db)
            .await?;
        return if assigned.is_some() {
            Ok(ReadScope::Blind)
        } else {
            Err(AppError::NotFound("申报书不存在".to_owned()))
        };
    }
    Err(AppError::Forbidden("只能查看本人负责的申报书".to_owned()))
}

fn ensure_owner_can_edit(user: &CurrentUser, app: &applications::Model) -> Result<(), AppError> {
    if app.principal_id != user.id {
        return Err(AppError::Forbidden("只能修改本人负责的申报书".to_owned()));
    }
    if matches!(
        app.status,
        ApplicationStatus::Draft
            | ApplicationStatus::DeptReturned
            | ApplicationStatus::OfficeReturned
    ) {
        Ok(())
    } else {
        Err(AppError::Conflict("当前状态不能修改申报书".to_owned()))
    }
}

async fn load_application(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<applications::Model, AppError> {
    applications::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报书不存在".to_owned()))
}

async fn load_attachment(
    db: &DatabaseConnection,
    application_id: Uuid,
    attachment_id: Uuid,
) -> Result<application_attachments::Model, AppError> {
    let row = application_attachments::Entity::find_by_id(attachment_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("附件不存在".to_owned()))?;
    if row.application_id != application_id {
        return Err(AppError::NotFound("附件不存在".to_owned()));
    }
    Ok(row)
}

fn parse_kind(raw: &str) -> Result<AttachmentKind, AppError> {
    match raw {
        "" | "supporting" => Ok(AttachmentKind::Supporting),
        "signature" => Ok(AttachmentKind::Signature),
        "other" => Ok(AttachmentKind::Other),
        _ => Err(invalid("kind", "附件类型无效")),
    }
}

fn kind_code(kind: &AttachmentKind) -> &'static str {
    match kind {
        AttachmentKind::Signature => "signature",
        AttachmentKind::Supporting => "supporting",
        AttachmentKind::Other => "other",
    }
}

fn to_view(row: application_attachments::Model) -> AttachmentView {
    AttachmentView {
        id: row.id,
        kind: kind_code(&row.kind).to_owned(),
        file_name: row.file_name,
        content_type: row.content_type,
        byte_size: row.byte_size,
        created_at: row.created_at.to_rfc3339(),
    }
}

fn invalid(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![crate::error::FieldError {
        field: field.to_owned(),
        message: message.to_owned(),
    }])
}

fn visible_kind(kind: &AttachmentKind, include_signature: bool) -> bool {
    include_signature || *kind != AttachmentKind::Signature
}

pub fn content_disposition(file_name: &str) -> String {
    let ascii: String = file_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let mut encoded = String::new();
    for byte in file_name.as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_') {
            encoded.push(*byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    let fallback = if ascii.is_empty() {
        "attachment".to_owned()
    } else {
        ascii
    };
    format!("attachment; filename=\"{fallback}\"; filename*=UTF-8''{encoded}")
}

#[cfg(test)]
mod tests {
    use super::{content_disposition, visible_kind};
    use entity::sea_orm_active_enums::AttachmentKind;

    #[test]
    fn expert_cannot_see_signature_kind() {
        assert!(!visible_kind(&AttachmentKind::Signature, false));
        assert!(visible_kind(&AttachmentKind::Supporting, false));
        assert!(visible_kind(&AttachmentKind::Signature, true));
    }

    #[test]
    fn download_name_keeps_utf8_filename() {
        let header = content_disposition("签字件.png");
        assert!(header.contains("filename*=UTF-8''"));
        assert!(header.contains("%E7%AD%BE%E5%AD%97%E4%BB%B6.png"));
    }
}
