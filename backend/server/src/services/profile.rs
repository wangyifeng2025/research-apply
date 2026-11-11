//! 当前登录用户的个人资料。新建申报书时从这里复制负责人快照。

use chrono::NaiveDate;
use entity::{departments, users};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::{AppError, FieldError};
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};
use crate::support::time::now;

#[derive(Debug, Deserialize)]
pub struct ProfileInput {
    pub gender: Option<String>,
    pub ethnicity: Option<String>,
    pub birth_date: Option<String>,
    pub admin_title: Option<String>,
    pub professional_title: Option<String>,
    pub research_expertise: Option<String>,
    pub education: Option<String>,
    pub degree: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProfileView {
    pub id: Uuid,
    pub username: String,
    pub real_name: String,
    pub employee_no: Option<String>,
    pub gender: Option<String>,
    pub ethnicity: Option<String>,
    pub birth_date: Option<String>,
    pub admin_title: Option<String>,
    pub professional_title: Option<String>,
    pub research_expertise: Option<String>,
    pub education: Option<String>,
    pub degree: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub department_name: Option<String>,
}

pub async fn show(state: &AppState, actor: &CurrentUser) -> Result<ProfileView, AppError> {
    let user = load_user(&state.db, actor.id).await?;
    to_view(&state.db, user).await
}

pub async fn update(
    state: &AppState,
    actor: &CurrentUser,
    input: ProfileInput,
) -> Result<ProfileView, AppError> {
    let existing = load_user(&state.db, actor.id).await?;
    let mut issues = Vec::new();
    let gender = merge_text(
        input.gender,
        existing.gender.clone(),
        8,
        "gender",
        "性别最多 8 个字",
        &mut issues,
    );
    let ethnicity = merge_text(
        input.ethnicity,
        existing.ethnicity.clone(),
        32,
        "ethnicity",
        "民族最多 32 个字",
        &mut issues,
    );
    let admin_title = merge_text(
        input.admin_title,
        existing.admin_title.clone(),
        64,
        "admin_title",
        "行政职务最多 64 个字",
        &mut issues,
    );
    let professional_title = merge_text(
        input.professional_title,
        existing.professional_title.clone(),
        64,
        "professional_title",
        "专业技术职务最多 64 个字",
        &mut issues,
    );
    let research_expertise = merge_text(
        input.research_expertise,
        existing.research_expertise.clone(),
        120,
        "research_expertise",
        "研究专长最多 120 个字",
        &mut issues,
    );
    let education = merge_text(
        input.education,
        existing.education.clone(),
        32,
        "education",
        "学历最多 32 个字",
        &mut issues,
    );
    let degree = merge_text(
        input.degree,
        existing.degree.clone(),
        32,
        "degree",
        "学位最多 32 个字",
        &mut issues,
    );
    let phone = merge_text(
        input.phone,
        existing.phone.clone(),
        20,
        "phone",
        "手机号码最多 20 位",
        &mut issues,
    );
    let birth_date = merge_birth(input.birth_date, existing.birth_date, &mut issues);
    let email = merge_email(input.email, existing.email.clone(), &mut issues);
    if !issues.is_empty() {
        return Err(AppError::Validation(issues));
    }
    if let Some(email) = email.as_deref() {
        let duplicated = users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .filter(users::Column::Id.ne(actor.id))
            .one(&state.db)
            .await?;
        if duplicated.is_some() {
            return Err(AppError::Conflict("邮箱已存在".to_owned()));
        }
    }

    let mut model: users::ActiveModel = existing.into();
    model.gender = Set(gender);
    model.ethnicity = Set(ethnicity);
    model.birth_date = Set(birth_date);
    model.admin_title = Set(admin_title);
    model.professional_title = Set(professional_title);
    model.research_expertise = Set(research_expertise);
    model.education = Set(education);
    model.degree = Set(degree);
    model.phone = Set(phone);
    model.email = Set(email);
    model.updated_at = Set(now());
    let txn = state.db.begin().await?;
    let saved = model.update(&txn).await?;
    record(
        &txn,
        AuditEntry {
            actor_id: Some(actor.id),
            action: "profile_updated",
            target_type: "user",
            target_id: Some(actor.id),
            detail: Some(json!({ "username": actor.username })),
            ip_address: None,
            user_agent: None,
        },
    )
    .await?;
    txn.commit().await?;
    to_view(&state.db, saved).await
}

async fn load_user(db: &sea_orm::DatabaseConnection, id: Uuid) -> Result<users::Model, AppError> {
    users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("登录已失效，请重新登录".to_owned()))
}

async fn to_view(
    db: &sea_orm::DatabaseConnection,
    user: users::Model,
) -> Result<ProfileView, AppError> {
    let department_name = match user.department_id {
        Some(department_id) => departments::Entity::find_by_id(department_id)
            .one(db)
            .await?
            .map(|department| department.name),
        None => None,
    };
    Ok(ProfileView {
        id: user.id,
        username: user.username,
        real_name: user.real_name,
        employee_no: user.employee_no,
        gender: user.gender,
        ethnicity: user.ethnicity,
        birth_date: user.birth_date.map(|date| date.format("%Y-%m").to_string()),
        admin_title: user.admin_title,
        professional_title: user.professional_title,
        research_expertise: user.research_expertise,
        education: user.education,
        degree: user.degree,
        phone: user.phone,
        email: user.email,
        department_name,
    })
}

fn merge_text(
    input: Option<String>,
    current: Option<String>,
    max: usize,
    field: &str,
    message: &str,
    issues: &mut Vec<FieldError>,
) -> Option<String> {
    let value = match input {
        Some(value) => blank_to_none(value),
        None => current,
    };
    if value
        .as_ref()
        .is_some_and(|text| text.chars().count() > max)
    {
        issues.push(FieldError {
            field: field.to_owned(),
            message: message.to_owned(),
        });
    }
    value
}

fn merge_email(
    input: Option<String>,
    current: Option<String>,
    issues: &mut Vec<FieldError>,
) -> Option<String> {
    let Some(value) = input else {
        return current;
    };
    let value = blank_to_none(value)?;
    let valid =
        value.chars().count() <= 120 && value.contains('@') && !value.contains(char::is_whitespace);
    if !valid {
        issues.push(FieldError {
            field: "email".to_owned(),
            message: "请填写有效邮箱".to_owned(),
        });
    }
    Some(value)
}

fn merge_birth(
    input: Option<String>,
    current: Option<NaiveDate>,
    issues: &mut Vec<FieldError>,
) -> Option<NaiveDate> {
    let Some(value) = input else {
        return current;
    };
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    match parse_birth(value) {
        Ok(date) => Some(date),
        Err(message) => {
            issues.push(FieldError {
                field: "birth_date".to_owned(),
                message: message.to_owned(),
            });
            current
        }
    }
}

fn parse_birth(raw: &str) -> Result<NaiveDate, &'static str> {
    let dashed = raw.replace(['.', '/', '年', '月'], "-").replace('日', "");
    let dashed = dashed.trim_matches('-');
    if let Ok(date) = NaiveDate::parse_from_str(dashed, "%Y-%m-%d") {
        return Ok(date);
    }
    if let Ok(date) = NaiveDate::parse_from_str(&format!("{dashed}-01"), "%Y-%m-%d") {
        return Ok(date);
    }
    Err("出生年月格式应为 YYYY-MM")
}

fn blank_to_none(value: String) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}
