use entity::{applications, departments, users};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::{CurrentUser, require_role};
use crate::error::{AppError, FieldError};
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;
use entity::sea_orm_active_enums::UserRole;

#[derive(Debug, Deserialize)]
pub struct DepartmentInput {
    pub name: String,
    pub code: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DepartmentView {
    pub id: Uuid,
    pub name: String,
    pub code: Option<String>,
    pub member_count: usize,
}

pub async fn list(db: &DatabaseConnection) -> Result<Vec<DepartmentView>, AppError> {
    let rows = departments::Entity::find()
        .find_with_related(users::Entity)
        .order_by_asc(departments::Column::Name)
        .all(db)
        .await?;
    Ok(rows
        .into_iter()
        .map(|(department, members)| DepartmentView {
            id: department.id,
            name: department.name,
            code: department.code,
            member_count: members.len(),
        })
        .collect())
}

pub async fn create(
    state: &AppState,
    actor: &CurrentUser,
    input: DepartmentInput,
) -> Result<DepartmentView, AppError> {
    require_role(actor, UserRole::Admin)?;
    let (name, code) = validate(input)?;
    ensure_unique(&state.db, &name, code.as_deref(), None).await?;
    let timestamp = now();
    let department = departments::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(name.clone()),
        code: Set(code.clone()),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    };
    let txn = state.db.begin().await?;
    let saved = department.insert(&txn).await?;
    record(
        &txn,
        AuditEntry {
            actor_id: Some(actor.id),
            action: "department_created",
            target_type: "department",
            target_id: Some(saved.id),
            detail: Some(json!({ "name": saved.name, "code": saved.code })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(view(saved, 0))
}

pub async fn update(
    state: &AppState,
    actor: &CurrentUser,
    id: Uuid,
    input: DepartmentInput,
) -> Result<DepartmentView, AppError> {
    require_role(actor, UserRole::Admin)?;
    let existing = departments::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("部门不存在".to_owned()))?;
    let (name, code) = validate(input)?;
    ensure_unique(&state.db, &name, code.as_deref(), Some(id)).await?;
    let member_count = users::Entity::find()
        .filter(users::Column::DepartmentId.eq(id))
        .count(&state.db)
        .await? as usize;
    let mut model: departments::ActiveModel = existing.into();
    model.name = Set(name.clone());
    model.code = Set(code.clone());
    model.updated_at = Set(now());
    let txn = state.db.begin().await?;
    let saved = model.update(&txn).await?;
    record(
        &txn,
        AuditEntry {
            actor_id: Some(actor.id),
            action: "department_updated",
            target_type: "department",
            target_id: Some(id),
            detail: Some(json!({ "name": name, "code": code })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(view(saved, member_count))
}

pub async fn delete(state: &AppState, actor: &CurrentUser, id: Uuid) -> Result<(), AppError> {
    require_role(actor, UserRole::Admin)?;
    let existing = departments::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("部门不存在".to_owned()))?;
    let member_count = users::Entity::find()
        .filter(users::Column::DepartmentId.eq(id))
        .count(&state.db)
        .await?;
    if member_count > 0 {
        return Err(AppError::Conflict(
            "该部门已有用户，不能删除；请先调整用户所属部门".to_owned(),
        ));
    }
    let application_count = applications::Entity::find()
        .filter(applications::Column::DepartmentId.eq(id))
        .count(&state.db)
        .await?;
    if application_count > 0 {
        return Err(AppError::Conflict(
            "该部门已关联申报书，不能删除".to_owned(),
        ));
    }
    let txn = state.db.begin().await?;
    departments::Entity::delete_by_id(id).exec(&txn).await?;
    record(
        &txn,
        AuditEntry {
            actor_id: Some(actor.id),
            action: "department_deleted",
            target_type: "department",
            target_id: Some(id),
            detail: Some(json!({ "name": existing.name, "code": existing.code })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    txn.commit().await?;
    Ok(())
}

fn view(model: departments::Model, member_count: usize) -> DepartmentView {
    DepartmentView {
        id: model.id,
        name: model.name,
        code: model.code,
        member_count,
    }
}

fn validate(input: DepartmentInput) -> Result<(String, Option<String>), AppError> {
    let name = input.name.trim().to_owned();
    let code = input.code.and_then(|value| {
        let value = value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    });
    let mut errors = Vec::new();
    if name.is_empty() {
        errors.push(FieldError {
            field: "name".to_owned(),
            message: "请输入部门名称".to_owned(),
        });
    } else if name.chars().count() > 100 {
        errors.push(FieldError {
            field: "name".to_owned(),
            message: "部门名称不能超过 100 个字符".to_owned(),
        });
    }
    if code
        .as_ref()
        .is_some_and(|value| value.chars().count() > 32)
    {
        errors.push(FieldError {
            field: "code".to_owned(),
            message: "部门编码不能超过 32 个字符".to_owned(),
        });
    }
    if !errors.is_empty() {
        return Err(AppError::Validation(errors));
    }
    Ok((name, code))
}

async fn ensure_unique(
    db: &DatabaseConnection,
    name: &str,
    code: Option<&str>,
    exclude_id: Option<Uuid>,
) -> Result<(), AppError> {
    let mut name_query = departments::Entity::find().filter(departments::Column::Name.eq(name));
    if let Some(id) = exclude_id {
        name_query = name_query.filter(departments::Column::Id.ne(id));
    }
    if name_query.one(db).await?.is_some() {
        return Err(AppError::Conflict("部门名称已存在".to_owned()));
    }
    if let Some(code) = code {
        let mut code_query = departments::Entity::find().filter(departments::Column::Code.eq(code));
        if let Some(id) = exclude_id {
            code_query = code_query.filter(departments::Column::Id.ne(id));
        }
        if code_query.one(db).await?.is_some() {
            return Err(AppError::Conflict("部门编码已存在".to_owned()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DepartmentInput, validate};

    #[test]
    fn department_name_is_required() {
        let result = validate(DepartmentInput {
            name: "  ".to_owned(),
            code: None,
        });
        assert!(result.is_err());
    }

    #[test]
    fn blank_department_code_becomes_none() {
        let result = validate(DepartmentInput {
            name: "科研处".to_owned(),
            code: Some("  ".to_owned()),
        });
        assert!(result.is_ok_and(|(_, code)| code.is_none()));
    }
}
