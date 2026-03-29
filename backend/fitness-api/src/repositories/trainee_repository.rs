use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{Trainee, TraineeMetric};

#[async_trait]
pub trait TraineeRepository: Send + Sync {
    async fn create_trainee(
        &self,
        coach_id: Uuid,
        display_name: &str,
        age: Option<i32>,
        email: Option<&str>,
        height_cm: Option<f64>,
        weight_kg: Option<f64>,
        notes: Option<&str>,
    ) -> AppResult<Trainee>;

    async fn list_by_coach(&self, coach_id: Uuid) -> AppResult<Vec<Trainee>>;

    async fn get_for_coach(&self, trainee_id: Uuid, coach_id: Uuid) -> AppResult<Trainee>;

    async fn update_trainee(
        &self,
        trainee_id: Uuid,
        coach_id: Uuid,
        display_name: Option<&str>,
        age: Option<i32>,
        email: Option<&str>,
        height_cm: Option<f64>,
        weight_kg: Option<f64>,
        notes: Option<&str>,
    ) -> AppResult<Trainee>;

    async fn delete_trainee(&self, trainee_id: Uuid, coach_id: Uuid) -> AppResult<()>;

    async fn log_metric(
        &self,
        trainee_id: Uuid,
        coach_id: Uuid,
        weight_kg: f64,
        height_cm: Option<f64>,
    ) -> AppResult<TraineeMetric>;

    async fn list_metrics(
        &self,
        trainee_id: Uuid,
        coach_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<TraineeMetric>>;
}

pub struct PgTraineeRepository {
    pool: PgPool,
}

impl PgTraineeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TraineeRepository for PgTraineeRepository {
    async fn create_trainee(
        &self,
        coach_id: Uuid,
        display_name: &str,
        age: Option<i32>,
        email: Option<&str>,
        height_cm: Option<f64>,
        weight_kg: Option<f64>,
        notes: Option<&str>,
    ) -> AppResult<Trainee> {
        let t = sqlx::query_as::<_, Trainee>(
            r#"
            INSERT INTO trainees (coach_id, display_name, age, email, height_cm, weight_kg, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, coach_id, display_name, age, email, height_cm, weight_kg, notes, created_at
            "#,
        )
        .bind(coach_id)
        .bind(display_name)
        .bind(age)
        .bind(email)
        .bind(height_cm)
        .bind(weight_kg)
        .bind(notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(t)
    }

    async fn list_by_coach(&self, coach_id: Uuid) -> AppResult<Vec<Trainee>> {
        let list = sqlx::query_as::<_, Trainee>(
            r#"
            SELECT id, coach_id, display_name, age, email, height_cm, weight_kg, notes, created_at
            FROM trainees
            WHERE coach_id = $1
            ORDER BY display_name ASC
            "#,
        )
        .bind(coach_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(list)
    }

    async fn get_for_coach(&self, trainee_id: Uuid, coach_id: Uuid) -> AppResult<Trainee> {
        let t = sqlx::query_as::<_, Trainee>(
            r#"
            SELECT id, coach_id, display_name, age, email, height_cm, weight_kg, notes, created_at
            FROM trainees
            WHERE id = $1 AND coach_id = $2
            "#,
        )
        .bind(trainee_id)
        .bind(coach_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::NotFound)?;

        Ok(t)
    }

    async fn update_trainee(
        &self,
        trainee_id: Uuid,
        coach_id: Uuid,
        display_name: Option<&str>,
        age: Option<i32>,
        email: Option<&str>,
        height_cm: Option<f64>,
        weight_kg: Option<f64>,
        notes: Option<&str>,
    ) -> AppResult<Trainee> {
        self.get_for_coach(trainee_id, coach_id).await?;

        let t = sqlx::query_as::<_, Trainee>(
            r#"
            UPDATE trainees SET
                display_name = COALESCE($3, display_name),
                age = COALESCE($4, age),
                email = COALESCE($5, email),
                height_cm = COALESCE($6, height_cm),
                weight_kg = COALESCE($7, weight_kg),
                notes = COALESCE($8, notes)
            WHERE id = $1 AND coach_id = $2
            RETURNING id, coach_id, display_name, age, email, height_cm, weight_kg, notes, created_at
            "#,
        )
        .bind(trainee_id)
        .bind(coach_id)
        .bind(display_name)
        .bind(age)
        .bind(email)
        .bind(height_cm)
        .bind(weight_kg)
        .bind(notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(t)
    }

    async fn delete_trainee(&self, trainee_id: Uuid, coach_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM trainees
            WHERE id = $1 AND coach_id = $2
            "#,
        )
        .bind(trainee_id)
        .bind(coach_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }
        Ok(())
    }

    async fn log_metric(
        &self,
        trainee_id: Uuid,
        coach_id: Uuid,
        weight_kg: f64,
        height_cm: Option<f64>,
    ) -> AppResult<TraineeMetric> {
        self.get_for_coach(trainee_id, coach_id).await?;

        let m = sqlx::query_as::<_, TraineeMetric>(
            r#"
            INSERT INTO trainee_metrics (trainee_id, weight_kg, height_cm)
            VALUES ($1, $2, $3)
            RETURNING id, trainee_id, weight_kg, height_cm, recorded_at
            "#,
        )
        .bind(trainee_id)
        .bind(weight_kg)
        .bind(height_cm)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"UPDATE trainees SET weight_kg = $2 WHERE id = $1 AND coach_id = $3"#,
        )
        .bind(trainee_id)
        .bind(weight_kg)
        .bind(coach_id)
        .execute(&self.pool)
        .await?;

        if let Some(h) = height_cm {
            sqlx::query(r#"UPDATE trainees SET height_cm = $2 WHERE id = $1 AND coach_id = $3"#)
                .bind(trainee_id)
                .bind(h)
                .bind(coach_id)
                .execute(&self.pool)
                .await?;
        }

        Ok(m)
    }

    async fn list_metrics(
        &self,
        trainee_id: Uuid,
        coach_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<TraineeMetric>> {
        self.get_for_coach(trainee_id, coach_id).await?;

        let list = sqlx::query_as::<_, TraineeMetric>(
            r#"
            SELECT id, trainee_id, weight_kg, height_cm, recorded_at
            FROM trainee_metrics
            WHERE trainee_id = $1
            ORDER BY recorded_at DESC
            LIMIT $2
            "#,
        )
        .bind(trainee_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(list)
    }
}
