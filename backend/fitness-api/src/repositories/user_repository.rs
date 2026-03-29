use std::borrow::Cow;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        first_name: &str,
        last_name: &str,
    ) -> AppResult<User>;
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
    async fn create_user(
        &self,
        email: &str,
        password_hash: &str,
        first_name: &str,
        last_name: &str,
    ) -> AppResult<User> {
        let row = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (email, password_hash, first_name, last_name, role)
            VALUES ($1, $2, $3, $4, 'coach')
            RETURNING id, email, first_name, last_name, role, password_hash, created_at
            "#,
        )
        .bind(email)
        .bind(password_hash)
        .bind(first_name)
        .bind(last_name)
        .fetch_one(&self.pool)
        .await;
        match row {
            Ok(u) => {
                // Seed default exercise library for each newly-created coach account.
                sqlx::query(
                    r#"
                    INSERT INTO exercises (user_id, name, muscle)
                    SELECT $1, name, muscle
                    FROM exercise_catalog
                    ON CONFLICT (user_id, name)
                    DO UPDATE SET muscle = EXCLUDED.muscle
                    "#,
                )
                .bind(u.id)
                .execute(&self.pool)
                .await?;
                Ok(u)
            }
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
            r#"SELECT id, email, first_name, last_name, role, password_hash, created_at FROM users WHERE email = $1"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(u)
    }

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        let u = sqlx::query_as::<_, User>(
            r#"SELECT id, email, first_name, last_name, role, password_hash, created_at FROM users WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(u)
    }
}
