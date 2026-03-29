use std::borrow::Cow;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, email: &str, password_hash: &str) -> AppResult<User>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
}

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create_user(&self, email: &str, password_hash: &str) -> AppResult<User> {
        let row = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash)
            VALUES ($1, $2)
            RETURNING id, email, password_hash, created_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await;
        match row {
            Ok(u) => Ok(u),
            Err(sqlx::Error::Database(ref e))
                if e.code() == Some(Cow::Borrowed("23505")) =>
            {
                Err(AppError::Conflict("email already registered".into()))
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let u = sqlx::query_as::<_, User>(
            r#"SELECT id, email, password_hash, created_at FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(u)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let u = sqlx::query_as::<_, User>(
            r#"SELECT id, email, password_hash, created_at FROM users WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(u)
    }
}
