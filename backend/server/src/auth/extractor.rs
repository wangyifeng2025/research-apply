use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use entity::sea_orm_active_enums::UserRole;
use entity::user_roles;
use entity::users;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::config::AUTH_COOKIE;
use crate::error::AppError;
use crate::state::AppState;

use super::jwt::decode_subject;

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: uuid::Uuid,
    pub username: String,
    pub real_name: String,
    pub roles: Vec<UserRole>,
    pub department_id: Option<uuid::Uuid>,
}

impl CurrentUser {
    pub fn has_role(&self, role: UserRole) -> bool {
        self.roles.contains(&role)
    }
}

pub fn require_role(user: &CurrentUser, role: UserRole) -> Result<(), AppError> {
    if user.has_role(role) || user.has_role(UserRole::Admin) {
        Ok(())
    } else {
        Err(AppError::Forbidden("没有执行该操作的权限".to_owned()))
    }
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(infallible) => match infallible {},
        };
        let token = token_from_request(&jar, parts)?;
        let user_id = decode_subject(&token, &state.config.jwt_secret)?;
        load_user(&state.db, user_id).await
    }
}

fn token_from_request(jar: &CookieJar, parts: &Parts) -> Result<String, AppError> {
    if let Some(cookie) = jar.get(AUTH_COOKIE) {
        let value = cookie.value().trim();
        if !value.is_empty() {
            return Ok(value.to_owned());
        }
    }
    if let Some(header) = parts.headers.get(AUTHORIZATION) {
        let value = header.to_str().unwrap_or("");
        if let Some(token) = value.strip_prefix("Bearer ") {
            let token = token.trim();
            if !token.is_empty() {
                return Ok(token.to_owned());
            }
        }
    }
    Err(AppError::Unauthorized("请先登录".to_owned()))
}

pub async fn load_user(
    db: &sea_orm::DatabaseConnection,
    user_id: uuid::Uuid,
) -> Result<CurrentUser, AppError> {
    let user = users::Entity::find_by_id(user_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("登录已失效，请重新登录".to_owned()))?;
    if !user.is_active {
        return Err(AppError::Forbidden("账号已停用".to_owned()));
    }
    let roles = user_roles::Entity::find()
        .filter(user_roles::Column::UserId.eq(user.id))
        .all(db)
        .await?
        .into_iter()
        .map(|role| role.role)
        .collect();
    Ok(CurrentUser {
        id: user.id,
        username: user.username,
        real_name: user.real_name,
        roles,
        department_id: user.department_id,
    })
}

#[cfg(test)]
mod tests {
    use entity::sea_orm_active_enums::UserRole;
    use uuid::Uuid;

    use super::{CurrentUser, require_role};

    fn user(roles: Vec<UserRole>) -> CurrentUser {
        CurrentUser {
            id: Uuid::nil(),
            username: "demo".to_owned(),
            real_name: "演示".to_owned(),
            roles,
            department_id: None,
        }
    }

    #[test]
    fn mismatched_role_is_forbidden() {
        let teacher = user(vec![UserRole::Teacher]);
        assert!(require_role(&teacher, UserRole::ResearchOffice).is_err());
    }

    #[test]
    fn admin_satisfies_any_role() {
        let admin = user(vec![UserRole::Admin]);
        assert!(require_role(&admin, UserRole::Expert).is_ok());
    }
}
