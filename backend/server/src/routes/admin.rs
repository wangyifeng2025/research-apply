use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::CurrentUser;
use crate::error::AppError;
use crate::services::departments::{self, DepartmentInput, DepartmentView};
use crate::services::users::{
    self as admin_users, AdminUserView, CreateUserInput, ResetPasswordInput, UpdateUserInput,
    UserQuery,
};
use crate::state::AppState;

pub async fn departments(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let items = departments::list(&state.db).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn create_department(
    State(state): State<AppState>,
    actor: CurrentUser,
    Json(input): Json<DepartmentInput>,
) -> Result<(StatusCode, Json<DepartmentView>), AppError> {
    let department = departments::create(&state, &actor, input).await?;
    Ok((StatusCode::CREATED, Json(department)))
}

pub async fn update_department(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<DepartmentInput>,
) -> Result<Json<DepartmentView>, AppError> {
    Ok(Json(departments::update(&state, &actor, id, input).await?))
}

pub async fn delete_department(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    departments::delete(&state, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn users(
    State(state): State<AppState>,
    actor: CurrentUser,
    Query(query): Query<UserQuery>,
) -> Result<Json<Value>, AppError> {
    let items = admin_users::list(&state.db, &actor, query).await?;
    Ok(Json(json!({ "items": items })))
}

pub async fn create_user(
    State(state): State<AppState>,
    actor: CurrentUser,
    Json(input): Json<CreateUserInput>,
) -> Result<(StatusCode, Json<AdminUserView>), AppError> {
    let user = admin_users::create(&state, &actor, input).await?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateUserInput>,
) -> Result<Json<AdminUserView>, AppError> {
    Ok(Json(admin_users::update(&state, &actor, id, input).await?))
}

pub async fn reset_password(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
    Json(input): Json<ResetPasswordInput>,
) -> Result<StatusCode, AppError> {
    admin_users::reset_password(&state, &actor, id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(
    State(state): State<AppState>,
    actor: CurrentUser,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    admin_users::delete(&state, &actor, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
