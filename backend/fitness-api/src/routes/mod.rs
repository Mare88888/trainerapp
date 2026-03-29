use axum::routing::{get, post};
use axum::Router;

use crate::handlers;
use crate::AppState;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(handlers::health))
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/me", get(handlers::me))
        .route("/api/exercises", get(handlers::list_exercises))
        .route("/trainees", get(handlers::list_trainees).post(handlers::create_trainee))
        .route(
            "/trainees/{trainee_id}",
            get(handlers::trainee_details)
                .put(handlers::replace_trainee_profile)
                .delete(handlers::delete_trainee),
        )
        .route(
            "/trainees/{trainee_id}/workouts",
            get(handlers::list_trainee_workouts).post(handlers::create_workout_for_trainee),
        )
        .route("/workouts/{workout_id}", get(handlers::workout_detail))
        .route("/api/trainees", get(handlers::list_trainees).post(handlers::create_trainee))
        .route(
            "/api/trainees/{trainee_id}",
            get(handlers::trainee_dashboard)
                .patch(handlers::update_trainee)
                .delete(handlers::delete_trainee),
        )
        .route(
            "/api/trainees/{trainee_id}/metrics",
            post(handlers::log_trainee_metric),
        )
        .route(
            "/api/trainees/{trainee_id}/workouts",
            get(handlers::list_trainee_workouts).post(handlers::create_workout_for_trainee),
        )
        .route(
            "/api/workouts",
            get(handlers::list_workouts).post(handlers::create_workout),
        )
        .route("/api/workouts/{workout_id}", get(handlers::workout_detail))
        .route(
            "/api/workouts/{workout_id}/exercises",
            get(handlers::list_workout_exercises).post(handlers::add_workout_exercise),
        )
        .route(
            "/api/workout-exercises/{workout_exercise_id}/sets",
            post(handlers::add_set),
        )
        .route("/api/stats/volume", get(handlers::volume_stats))
        .route("/api/stats/prs", get(handlers::personal_records))
}
