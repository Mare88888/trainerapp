use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Workout {
    pub id: Uuid,
    pub user_id: Uuid,
    pub trainee_id: Option<Uuid>,
    pub title: String,
    pub notes: Option<String>,
    pub started_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWorkoutRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub notes: Option<String>,
    #[serde(default)]
    pub trainee_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTraineeWorkoutRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkoutDetailExercise {
    pub workout_exercise_id: Uuid,
    pub position: i32,
    pub exercise: ExerciseSummary,
    pub sets: Vec<SetSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExerciseSummary {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SetSummary {
    pub id: Uuid,
    pub set_number: i32,
    pub reps: i32,
    pub weight_kg: f64,
    pub is_warmup: bool,
    pub set_type: String,
}

#[derive(Debug, Serialize)]
pub struct WorkoutDetail {
    pub workout: Workout,
    pub exercises: Vec<WorkoutDetailExercise>,
}
