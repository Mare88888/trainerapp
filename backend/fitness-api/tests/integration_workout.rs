use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use fitness_api::{config::Config, create_app};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

fn test_config() -> Config {
    Config {
        database_url: "postgres://unused".into(),
        jwt_secret: "01234567890123456789012345678901".into(),
        jwt_expiration_hours: 24,
        host: "0.0.0.0".into(),
        port: 3000,
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn post_workouts_requires_authentication(pool: sqlx::PgPool) {
    let app = create_app(pool, test_config());

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workouts")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "title": "Push",
                        "notes": null
                    }))
                    .unwrap(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn post_workouts_creates_for_authenticated_user(pool: sqlx::PgPool) {
    let app = create_app(pool, test_config());

    let reg_body = serde_json::to_vec(&json!({
        "email": "lifters@test.example",
        "password": "hunter2222"
    }))
    .unwrap();

    let reg = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(reg_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(reg.status(), StatusCode::OK);
    let bytes = reg.into_body().collect().await.unwrap().to_bytes();
    let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let token = parsed["token"].as_str().unwrap().to_string();

    let wo_body = serde_json::to_vec(&json!({
        "title": "Leg day",
        "notes": "felt strong"
    }))
    .unwrap();

    let wo = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/workouts")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(wo_body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(wo.status(), StatusCode::OK);
    let bytes = wo.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["title"], "Leg day");
    assert_eq!(body["notes"], "felt strong");
}
