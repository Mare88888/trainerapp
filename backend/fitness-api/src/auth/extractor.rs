use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts},
};
use uuid::Uuid;

use crate::auth::verify_token;
use crate::error::AppError;
use crate::AppState;

#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    AppState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl std::future::Future<Output = Result<Self, Self::Rejection>> + Send {
        async move {
            let app = AppState::from_ref(state);
            let header_val = parts
                .headers
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .ok_or(AppError::Unauthorized)?;

            let token = header_val
                .strip_prefix("Bearer ")
                .ok_or(AppError::Unauthorized)?;

            let user_id = verify_token(token, &app.config.jwt_secret)?;
            Ok(AuthUser { id: user_id })
        }
    }
}
