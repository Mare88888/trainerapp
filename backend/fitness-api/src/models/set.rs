use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Set {
    pub id: Uuid,
    pub workout_exercise_id: Uuid,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
    pub is_warmup: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddSetRequest {
    #[validate(range(min = 0))]
    pub reps: i32,
    #[validate(range(min = 0.0))]
    pub weight_kg: f64,
    #[serde(default)]
    pub is_warmup: bool,
}
