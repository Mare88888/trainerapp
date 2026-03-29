use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::models::{
    AddExerciseToWorkoutRequest, AddExerciseToWorkoutResponse, AddSetRequest, CreateWorkoutRequest,
    Workout, WorkoutDetail,
};
use crate::repositories::{
    ExercisePrRow, ExerciseVolumeRow, TraineeRepository, WorkoutRepository,
};

pub struct WorkoutService {
    workouts: Arc<dyn WorkoutRepository>,
    trainees: Arc<dyn TraineeRepository>,
}

impl WorkoutService {
    pub fn new(
        workouts: Arc<dyn WorkoutRepository>,
        trainees: Arc<dyn TraineeRepository>,
    ) -> Self {
        Self { workouts, trainees }
    }

    pub async fn create_workout(
        &self,
        user_id: Uuid,
        req: CreateWorkoutRequest,
    ) -> AppResult<Workout> {
        req.validate().map_err(AppError::Validation)?;
        if let Some(tid) = req.trainee_id {
            self.trainees.get_for_coach(tid, user_id).await?;
        }
        self.workouts
            .create_workout(
                user_id,
                &req.title,
                req.notes.as_deref(),
                req.trainee_id,
            )
            .await
    }

    pub async fn add_exercise_to_workout(
        &self,
        user_id: Uuid,
        workout_id: Uuid,
        req: AddExerciseToWorkoutRequest,
    ) -> AppResult<AddExerciseToWorkoutResponse> {
        req.validate().map_err(AppError::Validation)?;
        self.workouts
            .assert_workout_owner(workout_id, user_id)
            .await?;

        let exercise_id = self
            .workouts
            .find_or_create_exercise(user_id, &req.name)
            .await?;

        let workout_exercise = self
            .workouts
            .add_workout_exercise(workout_id, exercise_id, req.notes.as_deref())
            .await?;

        Ok(AddExerciseToWorkoutResponse {
            workout_exercise,
            exercise_id,
        })
    }

    pub async fn add_set(
        &self,
        user_id: Uuid,
        workout_exercise_id: Uuid,
        req: AddSetRequest,
    ) -> AppResult<crate::models::Set> {
        req.validate().map_err(AppError::Validation)?;
        self.workouts
            .assert_workout_exercise_owner(workout_exercise_id, user_id)
            .await?;
        self.workouts.add_set(workout_exercise_id, &req).await
    }

    pub async fn list_workouts(
        &self,
        user_id: Uuid,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<WorkoutDetail>> {
        if let Some(tid) = trainee_id {
            self.trainees.get_for_coach(tid, user_id).await?;
        }
        self.workouts
            .list_workouts_detailed(user_id, trainee_id)
            .await
    }

    pub async fn volume_stats(
        &self,
        user_id: Uuid,
        exercise_id: Option<Uuid>,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<ExerciseVolumeRow>> {
        if let Some(tid) = trainee_id {
            self.trainees.get_for_coach(tid, user_id).await?;
        }
        self.workouts
            .volume_by_exercise(user_id, exercise_id, trainee_id)
            .await
    }

    pub async fn personal_records(
        &self,
        user_id: Uuid,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<ExercisePrRow>> {
        if let Some(tid) = trainee_id {
            self.trainees.get_for_coach(tid, user_id).await?;
        }
        self.workouts
            .personal_records(user_id, trainee_id)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;

    use crate::models::{Set, Trainee, TraineeMetric, WorkoutExercise};
    use crate::repositories::TraineeRepository;

    struct StubTraineeRepo;

    #[async_trait]
    impl TraineeRepository for StubTraineeRepo {
        async fn create_trainee(
            &self,
            _: Uuid,
            _: &str,
            _: Option<&str>,
            _: Option<f64>,
            _: Option<f64>,
            _: Option<&str>,
        ) -> AppResult<Trainee> {
            unreachable!()
        }

        async fn list_by_coach(&self, _: Uuid) -> AppResult<Vec<Trainee>> {
            unreachable!()
        }

        async fn get_for_coach(&self, _: Uuid, _: Uuid) -> AppResult<Trainee> {
            Err(AppError::NotFound)
        }

        async fn update_trainee(
            &self,
            _: Uuid,
            _: Uuid,
            _: Option<&str>,
            _: Option<&str>,
            _: Option<f64>,
            _: Option<f64>,
            _: Option<&str>,
        ) -> AppResult<Trainee> {
            unreachable!()
        }

        async fn log_metric(
            &self,
            _: Uuid,
            _: Uuid,
            _: f64,
            _: Option<f64>,
        ) -> AppResult<TraineeMetric> {
            unreachable!()
        }

        async fn list_metrics(
            &self,
            _: Uuid,
            _: Uuid,
            _: i64,
        ) -> AppResult<Vec<TraineeMetric>> {
            unreachable!()
        }
    }

    struct ValidationOnlyRepo;

    #[async_trait]
    impl WorkoutRepository for ValidationOnlyRepo {
        async fn create_workout(
            &self,
            _: Uuid,
            _: &str,
            _: Option<&str>,
            _: Option<Uuid>,
        ) -> AppResult<Workout> {
            panic!("repository should not be called when validation fails");
        }

        async fn assert_workout_owner(&self, _: Uuid, _: Uuid) -> AppResult<()> {
            unreachable!()
        }

        async fn find_or_create_exercise(&self, _: Uuid, _: &str) -> AppResult<Uuid> {
            unreachable!()
        }

        async fn add_workout_exercise(
            &self,
            _: Uuid,
            _: Uuid,
            _: Option<&str>,
        ) -> AppResult<WorkoutExercise> {
            unreachable!()
        }

        async fn assert_workout_exercise_owner(&self, _: Uuid, _: Uuid) -> AppResult<()> {
            unreachable!()
        }

        async fn add_set(&self, _: Uuid, _: &AddSetRequest) -> AppResult<Set> {
            unreachable!()
        }

        async fn list_workouts_detailed(
            &self,
            _: Uuid,
            _: Option<Uuid>,
        ) -> AppResult<Vec<WorkoutDetail>> {
            unreachable!()
        }

        async fn volume_by_exercise(
            &self,
            _: Uuid,
            _: Option<Uuid>,
            _: Option<Uuid>,
        ) -> AppResult<Vec<ExerciseVolumeRow>> {
            unreachable!()
        }

        async fn personal_records(
            &self,
            _: Uuid,
            _: Option<Uuid>,
        ) -> AppResult<Vec<ExercisePrRow>> {
            unreachable!()
        }

        async fn list_recent_sessions_for_trainee(
            &self,
            _: Uuid,
            _: Uuid,
            _: i64,
        ) -> AppResult<Vec<crate::models::WorkoutSessionSummary>> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn create_workout_fails_validation_before_repo() {
        let svc = WorkoutService::new(
            Arc::new(ValidationOnlyRepo),
            Arc::new(StubTraineeRepo),
        );
        let req = CreateWorkoutRequest {
            title: "".to_string(),
            notes: None,
            trainee_id: None,
        };
        let err = svc
            .create_workout(Uuid::nil(), req)
            .await
            .expect_err("expected validation error");
        assert!(matches!(err, AppError::Validation(_)));
    }

    struct FixedWorkoutRepo {
        workout: Workout,
    }

    #[async_trait]
    impl WorkoutRepository for FixedWorkoutRepo {
        async fn create_workout(
            &self,
            user_id: Uuid,
            title: &str,
            notes: Option<&str>,
            trainee_id: Option<Uuid>,
        ) -> AppResult<Workout> {
            Ok(Workout {
                id: self.workout.id,
                user_id,
                trainee_id,
                title: title.to_string(),
                notes: notes.map(String::from),
                started_at: self.workout.started_at,
                created_at: self.workout.created_at,
            })
        }

        async fn assert_workout_owner(&self, _: Uuid, _: Uuid) -> AppResult<()> {
            unreachable!()
        }

        async fn find_or_create_exercise(&self, _: Uuid, _: &str) -> AppResult<Uuid> {
            unreachable!()
        }

        async fn add_workout_exercise(
            &self,
            _: Uuid,
            _: Uuid,
            _: Option<&str>,
        ) -> AppResult<WorkoutExercise> {
            unreachable!()
        }

        async fn assert_workout_exercise_owner(&self, _: Uuid, _: Uuid) -> AppResult<()> {
            unreachable!()
        }

        async fn add_set(&self, _: Uuid, _: &AddSetRequest) -> AppResult<Set> {
            unreachable!()
        }

        async fn list_workouts_detailed(
            &self,
            _: Uuid,
            _: Option<Uuid>,
        ) -> AppResult<Vec<WorkoutDetail>> {
            unreachable!()
        }

        async fn volume_by_exercise(
            &self,
            _: Uuid,
            _: Option<Uuid>,
            _: Option<Uuid>,
        ) -> AppResult<Vec<ExerciseVolumeRow>> {
            unreachable!()
        }

        async fn personal_records(
            &self,
            _: Uuid,
            _: Option<Uuid>,
        ) -> AppResult<Vec<ExercisePrRow>> {
            unreachable!()
        }

        async fn list_recent_sessions_for_trainee(
            &self,
            _: Uuid,
            _: Uuid,
            _: i64,
        ) -> AppResult<Vec<crate::models::WorkoutSessionSummary>> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn create_workout_passes_valid_payload_to_repo() {
        let now = Utc::now();
        let expected_id = Uuid::new_v4();
        let repo = FixedWorkoutRepo {
            workout: Workout {
                id: expected_id,
                user_id: Uuid::nil(),
                trainee_id: None,
                title: "ignored".into(),
                notes: None,
                started_at: now,
                created_at: now,
            },
        };
        let svc = WorkoutService::new(Arc::new(repo), Arc::new(StubTraineeRepo));
        let uid = Uuid::new_v4();
        let req = CreateWorkoutRequest {
            title: "Leg day".into(),
            notes: Some("light".into()),
            trainee_id: None,
        };
        let w = svc.create_workout(uid, req).await.expect("ok");
        assert_eq!(w.user_id, uid);
        assert_eq!(w.title, "Leg day");
        assert_eq!(w.notes.as_deref(), Some("light"));
        assert_eq!(w.id, expected_id);
    }
}
