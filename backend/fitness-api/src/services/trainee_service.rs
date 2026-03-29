use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateTraineeRequest, LogTraineeMetricRequest, Trainee, TraineeMetric, UpdateTraineeRequest,
    WorkoutSessionSummary,
};
use crate::repositories::{ExercisePrRow, ExerciseVolumeRow, TraineeRepository, WorkoutRepository};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TraineeDashboard {
    pub trainee: Trainee,
    pub metrics: Vec<TraineeMetric>,
    pub recent_sessions: Vec<WorkoutSessionSummary>,
    pub volume_by_exercise: Vec<ExerciseVolumeRow>,
    pub personal_records: Vec<ExercisePrRow>,
}

pub struct TraineeService {
    trainees: Arc<dyn TraineeRepository>,
    workouts: Arc<dyn WorkoutRepository>,
}

impl TraineeService {
    pub fn new(
        trainees: Arc<dyn TraineeRepository>,
        workouts: Arc<dyn WorkoutRepository>,
    ) -> Self {
        Self { trainees, workouts }
    }

    pub async fn create_trainee(
        &self,
        coach_id: Uuid,
        req: CreateTraineeRequest,
    ) -> AppResult<Trainee> {
        req.validate().map_err(AppError::Validation)?;
        self.trainees
            .create_trainee(
                coach_id,
                &req.display_name,
                req.age,
                req.email.as_deref(),
                req.height_cm,
                req.weight_kg,
                req.notes.as_deref(),
            )
            .await
    }

    pub async fn list_trainees(&self, coach_id: Uuid) -> AppResult<Vec<Trainee>> {
        self.trainees.list_by_coach(coach_id).await
    }

    pub async fn update_trainee(
        &self,
        coach_id: Uuid,
        trainee_id: Uuid,
        req: UpdateTraineeRequest,
    ) -> AppResult<Trainee> {
        if let Some(ref n) = req.display_name {
            if n.trim().is_empty() {
                return Err(AppError::BadRequest(
                    "display_name cannot be empty".into(),
                ));
            }
        }
        self.trainees
            .update_trainee(
                trainee_id,
                coach_id,
                req.display_name.as_deref(),
                req.age,
                req.email.as_deref(),
                req.height_cm,
                req.weight_kg,
                req.notes.as_deref(),
            )
            .await
    }

    pub async fn get_trainee(&self, coach_id: Uuid, trainee_id: Uuid) -> AppResult<Trainee> {
        self.trainees.get_for_coach(trainee_id, coach_id).await
    }

    pub async fn delete_trainee(&self, coach_id: Uuid, trainee_id: Uuid) -> AppResult<()> {
        self.trainees.delete_trainee(trainee_id, coach_id).await
    }

    pub async fn log_metric(
        &self,
        coach_id: Uuid,
        trainee_id: Uuid,
        req: LogTraineeMetricRequest,
    ) -> AppResult<TraineeMetric> {
        req.validate().map_err(AppError::Validation)?;
        self.trainees
            .log_metric(trainee_id, coach_id, req.weight_kg, req.height_cm)
            .await
    }

    pub async fn dashboard(&self, coach_id: Uuid, trainee_id: Uuid) -> AppResult<TraineeDashboard> {
        let trainee = self.trainees.get_for_coach(trainee_id, coach_id).await?;
        let metrics = self.trainees.list_metrics(trainee_id, coach_id, 40).await?;
        let recent_sessions = self
            .workouts
            .list_recent_sessions_for_trainee(coach_id, trainee_id, 20)
            .await?;
        let volume_by_exercise = self
            .workouts
            .volume_by_exercise(coach_id, None, Some(trainee_id))
            .await?;
        let personal_records = self
            .workouts
            .personal_records(coach_id, Some(trainee_id))
            .await?;

        Ok(TraineeDashboard {
            trainee,
            metrics,
            recent_sessions,
            volume_by_exercise,
            personal_records,
        })
    }
}
