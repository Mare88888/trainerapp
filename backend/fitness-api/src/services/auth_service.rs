use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::auth::{hash_password, sign_token, verify_password};
use crate::config::Config;
use crate::error::{AppError, AppResult};
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserPublic};
use crate::repositories::UserRepository;

pub struct AuthService {
    users: Arc<dyn UserRepository>,
    config: Arc<Config>,
}

impl AuthService {
    pub fn new(users: Arc<dyn UserRepository>, config: Arc<Config>) -> Self {
        Self { users, config }
    }

    pub async fn register(&self, req: RegisterRequest) -> AppResult<AuthResponse> {
        req.validate().map_err(AppError::Validation)?;
        let hash = hash_password(&req.password)?;
        let user = self
            .users
            .create_user(&req.email, &hash)
            .await?;
        let token = sign_token(
            user.id,
            &self.config.jwt_secret,
            self.config.jwt_expiration_hours,
        )?;
        Ok(AuthResponse {
            token,
            user: UserPublic::from(user),
        })
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        req.validate().map_err(AppError::Validation)?;
        let user = self
            .users
            .find_by_email(&req.email)
            .await?
            .ok_or(AppError::Unauthorized)?;

        let ok = verify_password(&req.password, &user.password_hash)?;
        if !ok {
            return Err(AppError::Unauthorized);
        }

        let token = sign_token(
            user.id,
            &self.config.jwt_secret,
            self.config.jwt_expiration_hours,
        )?;
        Ok(AuthResponse {
            token,
            user: UserPublic::from(user),
        })
    }

    pub async fn me(&self, user_id: Uuid) -> AppResult<UserPublic> {
        let user = self
            .users
            .find_by_id(user_id)
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(UserPublic::from(user))
    }
}
