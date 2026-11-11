use std::collections::HashMap;

use entity::sea_orm_active_enums::UserRole;
use entity::{
    application_attachments, applications, departments, expert_assignments, reviews, user_roles,
    users,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, Condition, ConnectionTrait,
    DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::{CurrentUser, hash_password, require_role};
use crate::error::{AppError, FieldError};
use crate::services::auth::role_code;
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Deserialize)]
pub struct UserQuery {
    pub department_id: Option<Uuid>,
    pub role: Option<String>,
    pub keyword: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserInput {
    pub username: String,
    pub password: String,
    pub real_name: String,
    pub employee_no: Option<String>,
    pub gender: Option<String>,
    pub ethnicity: Option<String>,
    pub admin_title: Option<String>,
    pub professional_title: Option<String>,
    pub research_expertise: Option<String>,
    pub education: Option<String>,
    pub degree: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub department_id: Option<Uuid>,
    pub is_active: Option<bool>,
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserInput {
    pub username: Option<String>,
    pub real_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub employee_no: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub gender: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub ethnicity: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub admin_title: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub professional_title: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub research_expertise: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub education: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub degree: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub phone: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub email: Option<Option<String>>,
    #[serde(
        default,
        deserialize_with = "crate::support::nullable::deserialize_nullable"
    )]
    pub department_id: Option<Option<Uuid>>,
    pub is_active: Option<bool>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordInput {
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AdminUserView {
    pub id: Uuid,
    pub username: String,
    pub real_name: String,
    pub employee_no: Option<String>,
    pub gender: Option<String>,
    pub ethnicity: Option<String>,
    pub admin_title: Option<String>,
    pub professional_title: Option<String>,
    pub research_expertise: Option<String>,
    pub education: Option<String>,
    pub degree: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub department_id: Option<Uuid>,
    pub department_name: Option<String>,
    pub is_active: bool,
    pub roles: Vec<&'static str>,
}

pub async fn list(
    db: &DatabaseConnection,
    actor: &CurrentUser,
    query: UserQuery,
) -> Result<Vec<AdminUserView>, AppError> {
    require_role(actor, UserRole::Admin)?;
    let role = query.role.as_deref().map(parse_role).transpose()?;
    let mut select = users::Entity::find();
    if let Some(department_id) = query.department_id {
        select = select.filter(users::Column::DepartmentId.eq(department_id));
    }
    if let Some(keyword) = query
        .keyword
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        select = select.filter(
            Condition::any()
                .add(users::Column::Username.contains(keyword))
                .add(users::Column::RealName.contains(keyword))
                .add(users::Column::EmployeeNo.contains(keyword)),
        );
    }
    let rows = select
        .find_also_related(departments::Entity)
        .order_by_asc(users::Column::RealName)
        .all(db)
        .await?;
    let user_ids = rows.iter().map(|(user, _)| user.id).collect::<Vec<_>>();
    let role_rows = if user_ids.is_empty() {
        Vec::new()
    } else {
        user_roles::Entity::find()
            .filter(user_roles::Column::UserId.is_in(user_ids))
            .all(db)
            .await?
    };
    let mut roles_by_user: HashMap<Uuid, Vec<UserRole>> = HashMap::new();
    for row in role_rows {
        roles_by_user.entry(row.user_id).or_default().push(row.role);
    }
    Ok(rows
        .into_iter()
        .filter_map(|(user, department)| {
            let roles = roles_by_user.remove(&user.id).unwrap_or_default();
            let matches = role
                .as_ref()
                .is_none_or(|required| roles.contains(required));
            matches.then(|| view(user, department, roles))
        })
        .collect())
}

pub async fn create(
    state: &AppState,
    actor: &CurrentUser,
    input: CreateUserInput,
) -> Result<AdminUserView, AppError> {
    require_role(actor, UserRole::Admin)?;
    let username = required(input.username, "username", "请输入登录账号", 64)?;
    let real_name = required(input.real_name, "real_name", "请输入姓名", 64)?;
    validate_password(&input.password)?;
    let roles = parse_roles(&input.roles)?;
    ensure_department(&state.db, input.department_id).await?;
    let employee_no = optional(input.employee_no);
    let email = optional(input.email);
    ensure_unique(
        &state.db,
        &username,
        employee_no.as_deref(),
        email.as_deref(),
        None,
    )
    .await?;
    let password_hash = hash_password(&input.password)?;
    let id = Uuid::new_v4();
    let timestamp = now();
    let model = users::ActiveModel {
        id: Set(id),
        username: Set(username.clone()),
        password_hash: Set(password_hash),
        real_name: Set(real_name.clone()),
        employee_no: Set(employee_no),
        birth_date: Set(None),
        gender: Set(optional(input.gender)),
        ethnicity: Set(optional(input.ethnicity)),
        admin_title: Set(optional(input.admin_title)),
        professional_title: Set(optional(input.professional_title)),
        research_expertise: Set(optional(input.research_expertise)),
        education: Set(optional(input.education)),
        degree: Set(optional(input.degree)),
        phone: Set(optional(input.phone)),
        email: Set(email),
        department_id: Set(input.department_id),
        is_active: Set(input.is_active.unwrap_or(true)),
        created_at: Set(timestamp),
        updated_at: Set(timestamp),
    };
    let txn = state.db.begin().await?;
    model.insert(&txn).await?;
    replace_roles(&txn, id, &roles).await?;
    write_audit(
        &txn,
        actor,
        "user_created",
        id,
        json!({ "username": username, "real_name": real_name, "roles": role_codes(&roles) }),
    )
    .await?;
    txn.commit().await?;
    get(&state.db, id).await
}

pub async fn update(
    state: &AppState,
    actor: &CurrentUser,
    id: Uuid,
    input: UpdateUserInput,
) -> Result<AdminUserView, AppError> {
    require_role(actor, UserRole::Admin)?;
    let existing = users::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_owned()))?;
    let username = match input.username {
        Some(value) => required(value, "username", "请输入登录账号", 64)?,
        None => existing.username.clone(),
    };
    let real_name = match input.real_name {
        Some(value) => required(value, "real_name", "请输入姓名", 64)?,
        None => existing.real_name.clone(),
    };
    let employee_no = patched_optional(input.employee_no, existing.employee_no.clone());
    let email = patched_optional(input.email, existing.email.clone());
    let department_id = input.department_id.unwrap_or(existing.department_id);
    let is_active = input.is_active.unwrap_or(existing.is_active);
    let roles = match input.roles {
        Some(values) => parse_roles(&values)?,
        None => load_roles(&state.db, id).await?,
    };
    if actor.id == id && !is_active {
        return Err(AppError::Conflict("不能停用当前登录账号".to_owned()));
    }
    if actor.id == id && !roles.contains(&UserRole::Admin) {
        return Err(AppError::Conflict(
            "不能移除当前登录账号的管理员角色".to_owned(),
        ));
    }
    ensure_department(&state.db, department_id).await?;
    ensure_unique(
        &state.db,
        &username,
        employee_no.as_deref(),
        email.as_deref(),
        Some(id),
    )
    .await?;
    let mut model: users::ActiveModel = existing.into();
    model.username = Set(username.clone());
    model.real_name = Set(real_name.clone());
    model.employee_no = Set(employee_no);
    model.gender = Set(patched_optional(
        input.gender,
        model.gender.take().unwrap_or(None),
    ));
    model.ethnicity = Set(patched_optional(
        input.ethnicity,
        model.ethnicity.take().unwrap_or(None),
    ));
    model.admin_title = Set(patched_optional(
        input.admin_title,
        model.admin_title.take().unwrap_or(None),
    ));
    model.professional_title = Set(patched_optional(
        input.professional_title,
        model.professional_title.take().unwrap_or(None),
    ));
    model.research_expertise = Set(patched_optional(
        input.research_expertise,
        model.research_expertise.take().unwrap_or(None),
    ));
    model.education = Set(patched_optional(
        input.education,
        model.education.take().unwrap_or(None),
    ));
    model.degree = Set(patched_optional(
        input.degree,
        model.degree.take().unwrap_or(None),
    ));
    model.phone = Set(patched_optional(
        input.phone,
        model.phone.take().unwrap_or(None),
    ));
    model.email = Set(email);
    model.department_id = Set(department_id);
    model.is_active = Set(is_active);
    model.updated_at = Set(now());
    let txn = state.db.begin().await?;
    model.update(&txn).await?;
    replace_roles(&txn, id, &roles).await?;
    write_audit(
        &txn,
        actor,
        "user_updated",
        id,
        json!({ "username": username, "real_name": real_name, "is_active": is_active, "roles": role_codes(&roles) }),
    )
    .await?;
    txn.commit().await?;
    get(&state.db, id).await
}

pub async fn reset_password(
    state: &AppState,
    actor: &CurrentUser,
    id: Uuid,
    input: ResetPasswordInput,
) -> Result<(), AppError> {
    require_role(actor, UserRole::Admin)?;
    validate_password(&input.password)?;
    let existing = users::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_owned()))?;
    let password_hash = hash_password(&input.password)?;
    let mut model: users::ActiveModel = existing.into();
    model.password_hash = Set(password_hash);
    model.updated_at = Set(now());
    let txn = state.db.begin().await?;
    model.update(&txn).await?;
    write_audit(&txn, actor, "user_password_reset", id, json!({})).await?;
    txn.commit().await?;
    Ok(())
}

pub async fn delete(state: &AppState, actor: &CurrentUser, id: Uuid) -> Result<(), AppError> {
    require_role(actor, UserRole::Admin)?;
    let existing = users::Entity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_owned()))?;
    if actor.id == id {
        return Err(AppError::Conflict("不能删除当前登录账号".to_owned()));
    }
    let has_applications = applications::Entity::find()
        .filter(
            Condition::any()
                .add(applications::Column::PrincipalId.eq(id))
                .add(applications::Column::ConcludedBy.eq(id)),
        )
        .count(&state.db)
        .await?;
    let has_attachments = application_attachments::Entity::find()
        .filter(application_attachments::Column::UploadedBy.eq(id))
        .count(&state.db)
        .await?;
    let has_assignments = expert_assignments::Entity::find()
        .filter(
            Condition::any()
                .add(expert_assignments::Column::ExpertId.eq(id))
                .add(expert_assignments::Column::AssignedBy.eq(id)),
        )
        .count(&state.db)
        .await?;
    let has_reviews = reviews::Entity::find()
        .filter(reviews::Column::ReviewerId.eq(id))
        .count(&state.db)
        .await?;
    if has_applications > 0 || has_attachments > 0 || has_assignments > 0 || has_reviews > 0 {
        return Err(AppError::Conflict(
            "该用户已关联申报或审核数据，不能删除；请改用停用".to_owned(),
        ));
    }
    let txn = state.db.begin().await?;
    users::Entity::delete_by_id(id).exec(&txn).await?;
    write_audit(
        &txn,
        actor,
        "user_deleted",
        id,
        json!({ "username": existing.username, "real_name": existing.real_name }),
    )
    .await?;
    txn.commit().await?;
    Ok(())
}

async fn get(db: &DatabaseConnection, id: Uuid) -> Result<AdminUserView, AppError> {
    let (user, department) = users::Entity::find_by_id(id)
        .find_also_related(departments::Entity)
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("用户不存在".to_owned()))?;
    let roles = load_roles(db, id).await?;
    Ok(view(user, department, roles))
}

fn view(
    user: users::Model,
    department: Option<departments::Model>,
    mut roles: Vec<UserRole>,
) -> AdminUserView {
    roles.sort_by_key(|role| role_code(role.clone()));
    AdminUserView {
        id: user.id,
        username: user.username,
        real_name: user.real_name,
        employee_no: user.employee_no,
        gender: user.gender,
        ethnicity: user.ethnicity,
        admin_title: user.admin_title,
        professional_title: user.professional_title,
        research_expertise: user.research_expertise,
        education: user.education,
        degree: user.degree,
        phone: user.phone,
        email: user.email,
        department_id: user.department_id,
        department_name: department.map(|value| value.name),
        is_active: user.is_active,
        roles: role_codes(&roles),
    }
}

fn role_codes(roles: &[UserRole]) -> Vec<&'static str> {
    roles.iter().cloned().map(role_code).collect()
}

async fn load_roles(db: &DatabaseConnection, id: Uuid) -> Result<Vec<UserRole>, AppError> {
    Ok(user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(id))
        .all(db)
        .await?
        .into_iter()
        .map(|row| row.role)
        .collect())
}

async fn replace_roles<C>(db: &C, id: Uuid, roles: &[UserRole]) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    user_roles::Entity::delete_many()
        .filter(user_roles::Column::UserId.eq(id))
        .exec(db)
        .await?;
    for role in roles {
        user_roles::ActiveModel {
            user_id: Set(id),
            role: Set(role.clone()),
        }
        .insert(db)
        .await?;
    }
    Ok(())
}

fn parse_roles(values: &[String]) -> Result<Vec<UserRole>, AppError> {
    if values.is_empty() {
        return Err(invalid("roles", "请至少分配一个角色"));
    }
    let mut roles = Vec::new();
    for value in values {
        let role = parse_role(value)?;
        if !roles.contains(&role) {
            roles.push(role);
        }
    }
    Ok(roles)
}

fn parse_role(value: &str) -> Result<UserRole, AppError> {
    match value {
        "teacher" => Ok(UserRole::Teacher),
        "department_reviewer" => Ok(UserRole::DepartmentReviewer),
        "research_office" => Ok(UserRole::ResearchOffice),
        "expert" => Ok(UserRole::Expert),
        "admin" => Ok(UserRole::Admin),
        _ => Err(invalid("role", "未知用户角色")),
    }
}

fn validate_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < 8 {
        return Err(invalid("password", "密码至少需要 8 个字符"));
    }
    Ok(())
}

fn required(value: String, field: &str, message: &str, max: usize) -> Result<String, AppError> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        return Err(invalid(field, message));
    }
    if value.chars().count() > max {
        return Err(invalid(field, "内容过长"));
    }
    Ok(value)
}

fn optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let value = value.trim().to_owned();
        (!value.is_empty()).then_some(value)
    })
}

fn patched_optional(input: Option<Option<String>>, current: Option<String>) -> Option<String> {
    match input {
        Some(value) => optional(value),
        None => current,
    }
}

async fn ensure_department(db: &DatabaseConnection, id: Option<Uuid>) -> Result<(), AppError> {
    if let Some(id) = id {
        if departments::Entity::find_by_id(id).one(db).await?.is_none() {
            return Err(invalid("department_id", "所选部门不存在"));
        }
    }
    Ok(())
}

async fn ensure_unique(
    db: &DatabaseConnection,
    username: &str,
    employee_no: Option<&str>,
    email: Option<&str>,
    exclude_id: Option<Uuid>,
) -> Result<(), AppError> {
    ensure_field_unique(
        db,
        users::Column::Username,
        username,
        exclude_id,
        "登录账号已存在",
    )
    .await?;
    if let Some(employee_no) = employee_no {
        ensure_field_unique(
            db,
            users::Column::EmployeeNo,
            employee_no,
            exclude_id,
            "工号已存在",
        )
        .await?;
    }
    if let Some(email) = email {
        ensure_field_unique(db, users::Column::Email, email, exclude_id, "邮箱已存在").await?;
    }
    Ok(())
}

async fn ensure_field_unique(
    db: &DatabaseConnection,
    column: users::Column,
    value: &str,
    exclude_id: Option<Uuid>,
    message: &str,
) -> Result<(), AppError> {
    let mut query = users::Entity::find().filter(column.eq(value));
    if let Some(id) = exclude_id {
        query = query.filter(users::Column::Id.ne(id));
    }
    if query.one(db).await?.is_some() {
        return Err(AppError::Conflict(message.to_owned()));
    }
    Ok(())
}

async fn write_audit<C>(
    db: &C,
    actor: &CurrentUser,
    action: &'static str,
    target_id: Uuid,
    detail: Value,
) -> Result<(), AppError>
where
    C: ConnectionTrait,
{
    record(
        db,
        AuditEntry {
            actor_id: Some(actor.id),
            action,
            target_type: "user",
            target_id: Some(target_id),
            detail: Some(detail),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
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
    use entity::sea_orm_active_enums::UserRole;

    use super::{parse_role, parse_roles, validate_password};

    #[test]
    fn parses_teacher_and_expert_roles() {
        let roles = parse_roles(&["teacher".to_owned(), "expert".to_owned()]);
        assert!(roles.is_ok_and(|roles| roles == vec![UserRole::Teacher, UserRole::Expert]));
    }

    #[test]
    fn rejects_unknown_role() {
        assert!(parse_role("superuser").is_err());
    }

    #[test]
    fn rejects_empty_roles() {
        assert!(parse_roles(&[]).is_err());
    }

    #[test]
    fn rejects_short_password() {
        assert!(validate_password("short").is_err());
    }
}
