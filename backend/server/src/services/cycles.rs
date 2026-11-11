use chrono::{DateTime, FixedOffset};
use entity::application_cycles;
use entity::sea_orm_active_enums::{CycleStatus, UserRole};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::{CurrentUser, require_role};
use crate::error::{AppError, FieldError};
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Deserialize)]
pub struct CreateCycleInput {
    pub cycle_year: i32,
    pub title: String,
    pub opens_at: Option<DateTime<FixedOffset>>,
    pub closes_at: Option<DateTime<FixedOffset>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCycleInput {
    pub cycle_year: Option<i32>,
    pub title: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub opens_at: Option<Option<DateTime<FixedOffset>>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub closes_at: Option<Option<DateTime<FixedOffset>>>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CycleView {
    pub id: Uuid,
    pub cycle_year: i32,
    pub title: String,
    pub opens_at: Option<String>,
    pub closes_at: Option<String>,
    pub status: &'static str,
}

pub async fn list(
    db: &DatabaseConnection,
    actor: &CurrentUser,
) -> Result<Vec<CycleView>, AppError> {
    require_role(actor, UserRole::ResearchOffice)?;
    let cycles = application_cycles::Entity::find()
        .order_by_desc(application_cycles::Column::CycleYear)
        .order_by_desc(application_cycles::Column::CreatedAt)
        .all(db)
        .await?;
    Ok(cycles.into_iter().map(view).collect())
}

pub async fn current(db: &DatabaseConnection) -> Result<Option<CycleView>, AppError> {
    Ok(find_current_open(db).await?.map(view))
}

pub async fn create(
    state: &AppState,
    actor: &CurrentUser,
    input: CreateCycleInput,
) -> Result<CycleView, AppError> {
    require_role(actor, UserRole::ResearchOffice)?;
    let title = validate_title(input.title)?;
    validate_window(input.opens_at, input.closes_at)?;
    let status = parse_status(input.status.as_deref().unwrap_or("draft"))?;
    ensure_unique(&state.db, input.cycle_year, &title, None).await?;
    let timestamp = now();
    let model = application_cycles::ActiveModel {
        id: Set(Uuid::new_v4()),
        cycle_year: Set(input.cycle_year),
        title: Set(title.clone()),
        opens_at: Set(input.opens_at),
        closes_at: Set(input.closes_at),
        status: Set(status),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    };
    let txn = state.db.begin().await?;
    let saved = model.insert(&txn).await?;
    record(
        &txn,
        AuditEntry {
            actor_id: Some(actor.id),
            action: "cycle_created",
            target_type: "application_cycle",
            target_id: Some(saved.id),
            detail: Some(json!({ "cycle_year": saved.cycle_year, "title": title, "status": status_code(&saved.status) })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(view(saved))
}

pub async fn update(
    state: &AppState,
    actor: &CurrentUser,
    id: Uuid,
    input: UpdateCycleInput,
) -> Result<CycleView, AppError> {
    require_role(actor, UserRole::ResearchOffice)?;
    let existing = application_cycles::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报批次不存在".to_owned()))?;
    let cycle_year = input.cycle_year.unwrap_or(existing.cycle_year);
    let title = match input.title {
        Some(title) => validate_title(title)?,
        None => existing.title.clone(),
    };
    let opens_at = input.opens_at.unwrap_or(existing.opens_at);
    let closes_at = input.closes_at.unwrap_or(existing.closes_at);
    let status = match input.status {
        Some(status) => parse_status(&status)?,
        None => existing.status.clone(),
    };
    validate_window(opens_at, closes_at)?;
    ensure_unique(&state.db, cycle_year, &title, Some(id)).await?;
    let mut model: application_cycles::ActiveModel = existing.into();
    model.cycle_year = Set(cycle_year);
    model.title = Set(title.clone());
    model.opens_at = Set(opens_at);
    model.closes_at = Set(closes_at);
    model.status = Set(status.clone());
    model.updated_at = Set(now());
    let txn = state.db.begin().await?;
    let saved = model.update(&txn).await?;
    record(
        &txn,
        AuditEntry {
            actor_id: Some(actor.id),
            action: "cycle_updated",
            target_type: "application_cycle",
            target_id: Some(id),
            detail: Some(
                json!({ "cycle_year": cycle_year, "title": title, "status": status_code(&status) }),
            ),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(view(saved))
}

pub async fn require_current_open(
    db: &DatabaseConnection,
) -> Result<application_cycles::Model, AppError> {
    find_current_open(db)
        .await?
        .ok_or_else(|| AppError::Conflict("当前没有开放中的申报批次".to_owned()))
}

pub async fn ensure_open(db: &DatabaseConnection, id: Uuid) -> Result<(), AppError> {
    let cycle = application_cycles::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("申报批次不存在".to_owned()))?;
    let timestamp = now();
    if cycle.status == CycleStatus::Open
        && cycle.opens_at.is_none_or(|opens| opens <= timestamp)
        && cycle.closes_at.is_none_or(|closes| timestamp <= closes)
    {
        return Ok(());
    }
    Err(AppError::Conflict(
        "该申报书所属批次未开放或已截止".to_owned(),
    ))
}

async fn find_current_open(
    db: &DatabaseConnection,
) -> Result<Option<application_cycles::Model>, AppError> {
    let timestamp = now();
    let cycles = application_cycles::Entity::find()
        .filter(application_cycles::Column::Status.eq(CycleStatus::Open))
        .order_by_desc(application_cycles::Column::CycleYear)
        .order_by_desc(application_cycles::Column::CreatedAt)
        .all(db)
        .await?;
    Ok(cycles.into_iter().find(|cycle| {
        cycle.opens_at.is_none_or(|opens| opens <= timestamp)
            && cycle.closes_at.is_none_or(|closes| timestamp <= closes)
    }))
}

fn view(model: application_cycles::Model) -> CycleView {
    CycleView {
        id: model.id,
        cycle_year: model.cycle_year,
        title: model.title,
        opens_at: model.opens_at.map(|value| value.to_rfc3339()),
        closes_at: model.closes_at.map(|value| value.to_rfc3339()),
        status: status_code(&model.status),
    }
}

fn status_code(status: &CycleStatus) -> &'static str {
    match status {
        CycleStatus::Draft => "draft",
        CycleStatus::Open => "open",
        CycleStatus::Closed => "closed",
    }
}

fn parse_status(value: &str) -> Result<CycleStatus, AppError> {
    match value {
        "draft" => Ok(CycleStatus::Draft),
        "open" => Ok(CycleStatus::Open),
        "closed" => Ok(CycleStatus::Closed),
        _ => Err(invalid("status", "未知批次状态")),
    }
}

fn validate_title(value: String) -> Result<String, AppError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(invalid("title", "请输入批次名称"));
    }
    if value.chars().count() > 120 {
        return Err(invalid("title", "批次名称不能超过 120 个字符"));
    }
    Ok(value)
}

fn validate_window(
    opens_at: Option<DateTime<FixedOffset>>,
    closes_at: Option<DateTime<FixedOffset>>,
) -> Result<(), AppError> {
    if opens_at
        .zip(closes_at)
        .is_some_and(|(opens, closes)| closes < opens)
    {
        return Err(invalid("closes_at", "截止时间不能早于开放时间"));
    }
    Ok(())
}

async fn ensure_unique(
    db: &DatabaseConnection,
    cycle_year: i32,
    title: &str,
    exclude_id: Option<Uuid>,
) -> Result<(), AppError> {
    let mut query = application_cycles::Entity::find()
        .filter(application_cycles::Column::CycleYear.eq(cycle_year))
        .filter(application_cycles::Column::Title.eq(title));
    if let Some(id) = exclude_id {
        query = query.filter(application_cycles::Column::Id.ne(id));
    }
    if query.one(db).await?.is_some() {
        return Err(AppError::Conflict("同一年度的批次名称已存在".to_owned()));
    }
    Ok(())
}

fn invalid(field: &str, message: &str) -> AppError {
    AppError::Validation(vec![FieldError {
        field: field.to_owned(),
        message: message.to_owned(),
    }])
}

#[cfg(test)]
mod tests {
    use chrono::{FixedOffset, TimeZone};

    use super::{parse_status, validate_window};

    #[test]
    fn rejects_deadline_before_opening() {
        let offset = FixedOffset::east_opt(8 * 3600).unwrap_or_else(|| panic!("无效时区偏移"));
        let opens = offset.with_ymd_and_hms(2026, 11, 2, 9, 0, 0).single();
        let closes = offset.with_ymd_and_hms(2026, 11, 1, 9, 0, 0).single();
        assert!(validate_window(opens, closes).is_err());
    }

    #[test]
    fn rejects_unknown_cycle_status() {
        assert!(parse_status("expired").is_err());
    }
}
