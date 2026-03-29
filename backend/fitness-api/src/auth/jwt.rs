use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    role: String,
    exp: i64,
}

#[derive(Debug, Clone)]
pub struct AuthClaims {
    pub user_id: Uuid,
    pub role: String,
}

pub fn sign_token(
    user_id: Uuid,
    role: &str,
    secret: &str,
    expiration_hours: i64,
) -> AppResult<String> {
    let exp = Utc::now() + Duration::hours(expiration_hours);
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: exp.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(e.to_string()))
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<AuthClaims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let user_id =
        Uuid::parse_str(&data.claims.sub).map_err(|_| AppError::Auth("invalid subject".into()))?;
    Ok(AuthClaims {
        user_id,
        role: data.claims.role,
    })
}
