use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum_extra::extract::CookieJar;
use serde::Deserialize;

use crate::auth::{clear_session_cookie, encode_session, session_cookie};
use crate::error::AppError;
use crate::services::auth::{self, AuthUser};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[derive(serde::Serialize)]
pub struct LoginResponse {
    user: AuthUser,
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    headers: HeaderMap,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
    let (_current, user) = auth::login(
        &state,
        &body.username,
        &body.password,
        Some(address),
        &headers,
    )
    .await?;
    let token = encode_session(user.id, &state.config.jwt_secret)?;
    let jar = jar.add(session_cookie(token, state.config.cookie_secure));
    Ok((jar, Json(LoginResponse { user })))
}

pub async fn logout(jar: CookieJar) -> CookieJar {
    jar.add(clear_session_cookie())
}

pub async fn me(
    State(state): State<AppState>,
    user: crate::auth::CurrentUser,
) -> Result<Json<LoginResponse>, AppError> {
    let user = auth::profile_for(&state.db, &user).await?;
    Ok(Json(LoginResponse { user }))
}
