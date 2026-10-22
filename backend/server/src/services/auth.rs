use std::net::SocketAddr;

use axum::http::HeaderMap;
use entity::departments;
use entity::sea_orm_active_enums::UserRole;
use entity::users;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Serialize;
use uuid::Uuid;

use crate::auth::{CurrentUser, load_user, verify_password};
use crate::error::AppError;
use crate::state::AppState;
use crate::support::audit::{AuditEntry, record};

#[derive(Debug, Serialize)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub real_name: String,
    pub roles: Vec<&'static str>,
    pub department_name: Option<String>,
}

pub async fn login(
    state: &AppState,
    username: &str,
    password: &str,
    address: Option<SocketAddr>,
    headers: &HeaderMap,
) -> Result<(CurrentUser, AuthUser), AppError> {
    let username = username.trim();
    if username.is_empty() || password.is_empty() {
        return Err(AppError::Validation(vec![crate::error::FieldError {
            field: "username".to_owned(),
            message: "请输入账号和密码".to_owned(),
        }]));
    }

    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(&state.db)
        .await?;

    let Some(user) = user else {
        write_login_audit(state, None, username, address, headers, "login_failed").await;
        return Err(AppError::Unauthorized("账号或密码错误".to_owned()));
    };

    if !verify_password(password, &user.password_hash)? {
        write_login_audit(
            state,
            Some(user.id),
            username,
            address,
            headers,
            "login_failed",
        )
        .await;
        return Err(AppError::Unauthorized("账号或密码错误".to_owned()));
    }
    if !user.is_active {
        write_login_audit(
            state,
            Some(user.id),
            username,
            address,
            headers,
            "login_denied",
        )
        .await;
        return Err(AppError::Forbidden("账号已停用".to_owned()));
    }

    let current = load_user(&state.db, user.id).await?;
    let profile = profile_for(&state.db, &current).await?;
    write_login_audit(
        state,
        Some(user.id),
        username,
        address,
        headers,
        "login_succeeded",
    )
    .await;
    Ok((current, profile))
}

pub async fn profile_for(
    db: &sea_orm::DatabaseConnection,
    user: &CurrentUser,
) -> Result<AuthUser, AppError> {
    let department_name = match user.department_id {
        Some(department_id) => departments::Entity::find_by_id(department_id)
            .one(db)
            .await?
            .map(|department| department.name),
        None => None,
    };
    Ok(AuthUser {
        id: user.id,
        username: user.username.clone(),
        real_name: user.real_name.clone(),
        roles: user.roles.iter().cloned().map(role_code).collect(),
        department_name,
    })
}

pub fn role_code(role: UserRole) -> &'static str {
    match role {
        UserRole::Teacher => "teacher",
        UserRole::DepartmentReviewer => "department_reviewer",
        UserRole::ResearchOffice => "research_office",
        UserRole::Expert => "expert",
        UserRole::Admin => "admin",
    }
}

async fn write_login_audit(
    state: &AppState,
    actor_id: Option<Uuid>,
    username: &str,
    address: Option<SocketAddr>,
    headers: &HeaderMap,
    action: &'static str,
) {
    let user_agent = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned);
    let entry = AuditEntry {
        actor_id,
        action,
        target_type: "user",
        target_id: actor_id,
        detail: Some(serde_json::json!({ "username": username })),
        ip_address: address.map(|value| value.ip().to_string()),
        user_agent,
    };
    if let Err(error) = record(&state.db, entry).await {
        tracing::error!(error = %error, "写入登录审计失败");
    }
}
