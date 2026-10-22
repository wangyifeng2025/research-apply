use argon2::Argon2;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|error| AppError::Internal(format!("密码哈希失败: {error}")))
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let parsed = PasswordHash::new(password_hash)
        .map_err(|error| AppError::Internal(format!("密码哈希无法解析: {error}")))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

#[cfg(test)]
mod tests {
    use super::{hash_password, verify_password};

    #[test]
    fn hashed_password_matches_only_the_original() {
        let hash = hash_password("research2026").unwrap_or_else(|error| panic!("{error}"));
        assert!(verify_password("research2026", &hash).unwrap_or_else(|error| panic!("{error}")));
        assert!(
            !verify_password("other-password", &hash).unwrap_or_else(|error| panic!("{error}"))
        );
    }
}
