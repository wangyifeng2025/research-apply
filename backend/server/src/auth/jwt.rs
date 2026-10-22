use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration as ChronoDuration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::{AUTH_COOKIE, SESSION_HOURS};
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    exp: u64,
}

pub fn encode_session(user_id: Uuid, secret: &str) -> Result<String, AppError> {
    let exp = Utc::now()
        .checked_add_signed(ChronoDuration::hours(SESSION_HOURS))
        .ok_or_else(|| AppError::Internal("会话过期时间无效".to_owned()))?;
    let claims = Claims {
        sub: user_id,
        exp: u64::try_from(exp.timestamp()).unwrap_or(0),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|error| AppError::Internal(format!("签发登录凭证失败: {error}")))
}

pub fn decode_subject(token: &str, secret: &str) -> Result<Uuid, AppError> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("登录已失效，请重新登录".to_owned()))?;
    Ok(data.claims.sub)
}

pub fn session_cookie(token: String, secure: bool) -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE, token))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .secure(secure)
        .max_age(cookie::time::Duration::hours(SESSION_HOURS))
        .build()
}

pub fn clear_session_cookie() -> Cookie<'static> {
    Cookie::build((AUTH_COOKIE, String::new()))
        .http_only(true)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(cookie::time::Duration::seconds(0))
        .build()
}

#[cfg(test)]
mod tests {
    use super::{decode_subject, encode_session};
    use uuid::Uuid;

    #[test]
    fn session_token_round_trips_the_user_id() {
        let user_id = Uuid::new_v4();
        let token =
            encode_session(user_id, "test-secret").unwrap_or_else(|error| panic!("{error}"));
        let subject =
            decode_subject(&token, "test-secret").unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(subject, user_id);
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let token =
            encode_session(Uuid::new_v4(), "test-secret").unwrap_or_else(|error| panic!("{error}"));
        assert!(decode_subject(&token, "other-secret").is_err());
    }
}
