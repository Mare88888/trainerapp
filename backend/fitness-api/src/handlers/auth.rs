use axum::{extract::State, Json};

use crate::auth::AuthUser;
use crate::error::AppResult;
use crate::models::{AuthResponse, LoginRequest, RegisterRequest, UserPublic};
use crate::AppState;

pub async fn me(auth: AuthUser, State(state): State<AppState>) -> AppResult<Json<UserPublic>> {
    let u = state.auth_service.me(auth.id).await?;
    Ok(Json(u))
}

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponse>> {
    let res = state.auth_service.register(req).await?;
    Ok(Json(res))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    let res = state.auth_service.login(req).await?;
    Ok(Json(res))
}
