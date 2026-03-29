use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Exercise {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub muscle: String,
    pub created_at: DateTime<Utc>,
}
