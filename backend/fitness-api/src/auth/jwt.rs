use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: i64,
}

pub fn sign_token(user_id: Uuid, secret: &str, expiration_hours: i64) -> AppResult<String> {
    let exp = Utc::now() + Duration::hours(expiration_hours);
    let claims = Claims {
        sub: user_id.to_string(),
        exp: exp.timestamp(),
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Auth(e.to_string()))
}

pub fn verify_token(token: &str, secret: &str) -> AppResult<Uuid> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized)?;

    let sub = data.claims.sub;
    Uuid::parse_str(&sub).map_err(|_| AppError::Auth("invalid subject".into()))
}
