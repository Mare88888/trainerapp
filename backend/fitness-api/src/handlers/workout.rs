use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::models::{
    AddExerciseToWorkoutRequest, AddExerciseToWorkoutResponse, AddSetRequest, CreateWorkoutRequest,
    Set, Workout, WorkoutDetail,
};
use crate::repositories::{ExercisePrRow, ExerciseVolumeRow};
use crate::AppState;

pub async fn create_workout(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateWorkoutRequest>,
) -> AppResult<Json<Workout>> {
    let w = state.workout_service.create_workout(auth.id, req).await?;
    Ok(Json(w))
}

pub async fn add_workout_exercise(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(workout_id): Path<Uuid>,
    Json(req): Json<AddExerciseToWorkoutRequest>,
) -> AppResult<Json<AddExerciseToWorkoutResponse>> {
    let res = state
        .workout_service
        .add_exercise_to_workout(auth.id, workout_id, req)
        .await?;
    Ok(Json(res))
}

pub async fn add_set(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(workout_exercise_id): Path<Uuid>,
    Json(req): Json<AddSetRequest>,
) -> AppResult<Json<Set>> {
    let s = state
        .workout_service
        .add_set(auth.id, workout_exercise_id, req)
        .await?;
    Ok(Json(s))
}

#[derive(Debug, Deserialize)]
pub struct TraineeFilterQuery {
    pub trainee_id: Option<Uuid>,
}

pub async fn list_workouts(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<TraineeFilterQuery>,
) -> AppResult<Json<Vec<WorkoutDetail>>> {
    let list = state
        .workout_service
        .list_workouts(auth.id, q.trainee_id)
        .await?;
    Ok(Json(list))
}

#[derive(Debug, Deserialize)]
pub struct VolumeQuery {
    pub exercise_id: Option<Uuid>,
    pub trainee_id: Option<Uuid>,
}

pub async fn volume_stats(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<VolumeQuery>,
) -> AppResult<Json<Vec<ExerciseVolumeRow>>> {
    let rows = state
        .workout_service
        .volume_stats(auth.id, q.exercise_id, q.trainee_id)
        .await?;
    Ok(Json(rows))
}

pub async fn personal_records(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<TraineeFilterQuery>,
) -> AppResult<Json<Vec<ExercisePrRow>>> {
    let rows = state
        .workout_service
        .personal_records(auth.id, q.trainee_id)
        .await?;
    Ok(Json(rows))
}
