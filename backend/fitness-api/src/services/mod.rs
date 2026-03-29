mod auth_service;
mod trainee_service;
mod workout_service;

pub use auth_service::AuthService;
pub use trainee_service::{TraineeDashboard, TraineeService};
pub use workout_service::WorkoutService;
