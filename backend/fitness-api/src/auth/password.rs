use crate::error::{AppError, AppResult};

const BCRYPT_COST: u32 = 12;

pub fn hash_password(password: &str) -> AppResult<String> {
    bcrypt::hash(password, BCRYPT_COST).map_err(|e| AppError::Internal(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    bcrypt::verify(password, hash).map_err(|e| AppError::Internal(e.to_string()))
}
