use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Trainee {
    pub id: Uuid,
    pub coach_id: Uuid,
    pub display_name: String,
    pub email: Option<String>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTraineeRequest {
    #[validate(length(min = 1, max = 255))]
    pub display_name: String,
    pub email: Option<String>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTraineeRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub height_cm: Option<f64>,
    pub weight_kg: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct TraineeMetric {
    pub id: Uuid,
    pub trainee_id: Uuid,
    pub weight_kg: f64,
    pub height_cm: Option<f64>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LogTraineeMetricRequest {
    #[validate(range(min = 0.01, max = 500.0))]
    pub weight_kg: f64,
    pub height_cm: Option<f64>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct WorkoutSessionSummary {
    pub id: Uuid,
    pub title: String,
    pub started_at: DateTime<Utc>,
}
