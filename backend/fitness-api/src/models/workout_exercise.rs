use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WorkoutExercise {
    pub id: Uuid,
    pub workout_id: Uuid,
    pub exercise_id: Uuid,
    pub position: i32,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AddExerciseToWorkoutRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddExerciseToWorkoutResponse {
    pub workout_exercise: WorkoutExercise,
    pub exercise_id: Uuid,
}
