pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;

use axum::Router;
use repositories::{
    PgTraineeRepository, PgUserRepository, PgWorkoutRepository, TraineeRepository, UserRepository,
    WorkoutRepository,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub use config::Config;
pub use error::AppError;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
    pub auth_service: Arc<services::AuthService>,
    pub workout_service: Arc<services::WorkoutService>,
    pub trainee_service: Arc<services::TraineeService>,
}

pub fn create_app(pool: PgPool, config: Config) -> Router {
    let config = Arc::new(config);
    let user_repo: Arc<dyn UserRepository> = Arc::new(PgUserRepository::new(pool.clone()));
    let workout_repo: Arc<dyn WorkoutRepository> =
        Arc::new(PgWorkoutRepository::new(pool.clone()));
    let trainee_repo: Arc<dyn TraineeRepository> =
        Arc::new(PgTraineeRepository::new(pool.clone()));

    let auth_service = Arc::new(services::AuthService::new(user_repo, config.clone()));
    let workout_service = Arc::new(services::WorkoutService::new(
        workout_repo.clone(),
        trainee_repo.clone(),
    ));
    let trainee_service = Arc::new(services::TraineeService::new(
        trainee_repo,
        workout_repo,
    ));

    let state = AppState {
        pool,
        config,
        auth_service,
        workout_service,
        trainee_service,
    };

    Router::new()
        .merge(routes::api_routes())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
