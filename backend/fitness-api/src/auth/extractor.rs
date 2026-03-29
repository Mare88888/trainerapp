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
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Coach,
}

impl Role {
    fn from_str(input: &str) -> Result<Self, AppError> {
        match input {
            "coach" => Ok(Self::Coach),
            _ => Err(AppError::Unauthorized),
        }
    }
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

            let claims = verify_token(token, &app.config.jwt_secret)?;
            Ok(AuthUser {
                id: claims.user_id,
                role: Role::from_str(&claims.role)?,
            })
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CoachUser {
    pub id: Uuid,
}

impl<S> FromRequestParts<S> for CoachUser
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
            let auth = AuthUser::from_request_parts(parts, state).await?;
            if auth.role == Role::Coach {
                Ok(CoachUser { id: auth.id })
            } else {
                Err(AppError::Forbidden)
            }
        }
    }
}
