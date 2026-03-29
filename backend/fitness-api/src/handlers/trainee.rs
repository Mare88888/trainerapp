use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::models::{CreateTraineeRequest, LogTraineeMetricRequest, Trainee, UpdateTraineeRequest};
use crate::services::TraineeDashboard;
use crate::AppState;

pub async fn list_trainees(
    auth: AuthUser,
    State(state): State<AppState>,
) -> AppResult<Json<Vec<Trainee>>> {
    let list = state.trainee_service.list_trainees(auth.id).await?;
    Ok(Json(list))
}

pub async fn create_trainee(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateTraineeRequest>,
) -> AppResult<Json<Trainee>> {
    let t = state.trainee_service.create_trainee(auth.id, req).await?;
    Ok(Json(t))
}

pub async fn trainee_dashboard(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(trainee_id): Path<Uuid>,
) -> AppResult<Json<TraineeDashboard>> {
    let d = state
        .trainee_service
        .dashboard(auth.id, trainee_id)
        .await?;
    Ok(Json(d))
}

pub async fn update_trainee(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(trainee_id): Path<Uuid>,
    Json(req): Json<UpdateTraineeRequest>,
) -> AppResult<Json<Trainee>> {
    let t = state
        .trainee_service
        .update_trainee(auth.id, trainee_id, req)
        .await?;
    Ok(Json(t))
}

pub async fn log_trainee_metric(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(trainee_id): Path<Uuid>,
    Json(req): Json<LogTraineeMetricRequest>,
) -> AppResult<Json<crate::models::TraineeMetric>> {
    let m = state
        .trainee_service
        .log_metric(auth.id, trainee_id, req)
        .await?;
    Ok(Json(m))
}
