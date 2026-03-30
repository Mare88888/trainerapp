use std::borrow::Cow;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{
    AddSetRequest, Exercise, ExerciseSummary, Set, SetSummary, Workout, WorkoutDetail,
    WorkoutDetailExercise, WorkoutExercise,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExerciseVolumeRow {
    pub exercise_id: Uuid,
    pub exercise_name: String,
    pub total_volume: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ExercisePrRow {
    pub exercise_id: Uuid,
    pub exercise_name: String,
    pub max_weight_kg: f64,
}

#[async_trait]
pub trait WorkoutRepository: Send + Sync {
    async fn create_workout(
        &self,
        user_id: Uuid,
        title: &str,
        notes: Option<&str>,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Workout>;

    async fn assert_workout_owner(&self, workout_id: Uuid, user_id: Uuid) -> AppResult<()>;

    async fn find_or_create_exercise(&self, user_id: Uuid, name: &str) -> AppResult<Uuid>;

    async fn add_workout_exercise(
        &self,
        workout_id: Uuid,
        exercise_id: Uuid,
        notes: Option<&str>,
    ) -> AppResult<WorkoutExercise>;

    async fn assert_workout_exercise_owner(
        &self,
        workout_exercise_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<()>;

    async fn add_set(
        &self,
        workout_exercise_id: Uuid,
        req: &AddSetRequest,
    ) -> AppResult<Set>;

    async fn update_set_type(&self, set_id: Uuid, user_id: Uuid, set_type: &str) -> AppResult<Set>;

    async fn delete_set(&self, set_id: Uuid, user_id: Uuid) -> AppResult<()>;

    async fn list_workouts_detailed(
        &self,
        user_id: Uuid,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<WorkoutDetail>>;

    async fn volume_by_exercise(
        &self,
        user_id: Uuid,
        exercise_id: Option<Uuid>,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<ExerciseVolumeRow>>;

    async fn personal_records(
        &self,
        user_id: Uuid,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<ExercisePrRow>>;

    async fn list_recent_sessions_for_trainee(
        &self,
        coach_id: Uuid,
        trainee_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<crate::models::WorkoutSessionSummary>>;

    async fn list_exercises(
        &self,
        user_id: Uuid,
        muscle: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<Exercise>>;
}

pub struct PgWorkoutRepository {
    pool: PgPool,
}

impl PgWorkoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkoutRepository for PgWorkoutRepository {
    async fn create_workout(
        &self,
        user_id: Uuid,
        title: &str,
        notes: Option<&str>,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Workout> {
        let w = sqlx::query_as::<_, Workout>(
            r#"
            INSERT INTO workouts (user_id, trainee_id, title, notes)
            VALUES ($1, $2, $3, $4)
            RETURNING id, user_id, trainee_id, title, notes, started_at, created_at
            "#,
        )
        .bind(user_id)
        .bind(trainee_id)
        .bind(title)
        .bind(notes)
        .fetch_one(&self.pool)
        .await?;

        Ok(w)
    }

    async fn assert_workout_owner(&self, workout_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let ok: bool = sqlx::query_scalar(
            r#"SELECT EXISTS(SELECT 1 FROM workouts WHERE id = $1 AND user_id = $2)"#,
        )
        .bind(workout_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        if ok {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    async fn find_or_create_exercise(&self, user_id: Uuid, name: &str) -> AppResult<Uuid> {
        let id: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO exercises (user_id, name)
            VALUES ($1, $2)
            ON CONFLICT (user_id, name) DO NOTHING
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(id) = id {
            return Ok(id);
        }

        let id: Uuid = sqlx::query_scalar(
            r#"SELECT id FROM exercises WHERE user_id = $1 AND name = $2"#,
        )
        .bind(user_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn add_workout_exercise(
        &self,
        workout_id: Uuid,
        exercise_id: Uuid,
        notes: Option<&str>,
    ) -> AppResult<WorkoutExercise> {
        let pos: i32 = sqlx::query_scalar(
            r#"SELECT COALESCE(MAX(position), -1) + 1 FROM workout_exercises WHERE workout_id = $1"#,
        )
        .bind(workout_id)
        .fetch_one(&self.pool)
        .await?;

        let row = sqlx::query_as::<_, WorkoutExercise>(
            r#"
            INSERT INTO workout_exercises (workout_id, exercise_id, position, notes)
            VALUES ($1, $2, $3, $4)
            RETURNING id, workout_id, exercise_id, position, notes, created_at
            "#,
        )
        .bind(workout_id)
        .bind(exercise_id)
        .bind(pos)
        .bind(notes)
        .fetch_one(&self.pool)
        .await;

        match row {
            Ok(we) => Ok(we),
            Err(sqlx::Error::Database(ref e))
                if e.code() == Some(Cow::Borrowed("23505")) =>
            {
                Err(AppError::Conflict(
                    "exercise already added to this workout".into(),
                ))
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn assert_workout_exercise_owner(
        &self,
        workout_exercise_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<()> {
        let ok: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM workout_exercises we
                INNER JOIN workouts w ON w.id = we.workout_id
                WHERE we.id = $1 AND w.user_id = $2
            )
            "#,
        )
        .bind(workout_exercise_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        if ok {
            Ok(())
        } else {
            Err(AppError::NotFound)
        }
    }

    async fn add_set(
        &self,
        workout_exercise_id: Uuid,
        req: &AddSetRequest,
    ) -> AppResult<Set> {
        let next: i32 = sqlx::query_scalar(
            r#"SELECT COALESCE(MAX(set_number), 0) + 1 FROM sets WHERE workout_exercise_id = $1"#,
        )
        .bind(workout_exercise_id)
        .fetch_one(&self.pool)
        .await?;

        let s = sqlx::query_as::<_, Set>(
            r#"
            INSERT INTO sets (workout_exercise_id, set_number, reps, weight_kg, is_warmup, set_type)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, workout_exercise_id, set_number, reps, weight_kg, is_warmup, set_type, created_at
            "#,
        )
        .bind(workout_exercise_id)
        .bind(next)
        .bind(req.reps)
        .bind(req.weight_kg)
        .bind(req.is_warmup)
        .bind(if req.is_warmup { "warmup" } else { "normal" })
        .fetch_one(&self.pool)
        .await?;

        Ok(s)
    }

    async fn update_set_type(&self, set_id: Uuid, user_id: Uuid, set_type: &str) -> AppResult<Set> {
        let s = sqlx::query_as::<_, Set>(
            r#"
            UPDATE sets s
            SET set_type = $3, is_warmup = ($3 = 'warmup')
            FROM workout_exercises we
            INNER JOIN workouts w ON w.id = we.workout_id
            WHERE s.id = $1
              AND s.workout_exercise_id = we.id
              AND w.user_id = $2
            RETURNING s.id, s.workout_exercise_id, s.set_number, s.reps, s.weight_kg, s.is_warmup, s.set_type, s.created_at
            "#,
        )
        .bind(set_id)
        .bind(user_id)
        .bind(set_type)
        .fetch_optional(&self.pool)
        .await?;

        s.ok_or(AppError::NotFound)
    }

    async fn delete_set(&self, set_id: Uuid, user_id: Uuid) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;

        // Lock the parent line so concurrent set deletes cannot leave numbering inconsistent.
        let we_id: Option<Uuid> = sqlx::query_scalar(
            r#"
            SELECT we.id
            FROM sets s
            INNER JOIN workout_exercises we ON we.id = s.workout_exercise_id
            INNER JOIN workouts w ON w.id = we.workout_id
            WHERE s.id = $1
              AND w.user_id = $2
            FOR UPDATE OF we
            "#,
        )
        .bind(set_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let Some(we_id) = we_id else {
            return Err(AppError::NotFound);
        };

        let deleted = sqlx::query(
            r#"
            DELETE FROM sets s
            USING workout_exercises we, workouts w
            WHERE s.id = $1
              AND s.workout_exercise_id = we.id
              AND we.workout_id = w.id
              AND w.user_id = $2
            "#,
        )
        .bind(set_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();

        if deleted == 0 {
            return Err(AppError::NotFound);
        }

        // Re-sequence remaining sets to 1..N after every delete.
        sqlx::query(
            r#"UPDATE sets SET set_number = set_number + 1000000 WHERE workout_exercise_id = $1"#,
        )
        .bind(we_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            WITH ranked AS (
                SELECT id, ROW_NUMBER() OVER (ORDER BY set_number) AS rn
                FROM sets
                WHERE workout_exercise_id = $1
            )
            UPDATE sets s
            SET set_number = r.rn
            FROM ranked r
            WHERE s.id = r.id
            "#,
        )
        .bind(we_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn list_workouts_detailed(
        &self,
        user_id: Uuid,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<WorkoutDetail>> {
        let workouts = if let Some(tid) = trainee_id {
            sqlx::query_as::<_, Workout>(
                r#"
                SELECT id, user_id, trainee_id, title, notes, started_at, created_at
                FROM workouts
                WHERE user_id = $1 AND trainee_id = $2
                ORDER BY started_at DESC
                "#,
            )
            .bind(user_id)
            .bind(tid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, Workout>(
                r#"
                SELECT id, user_id, trainee_id, title, notes, started_at, created_at
                FROM workouts
                WHERE user_id = $1
                ORDER BY started_at DESC
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        if workouts.is_empty() {
            return Ok(vec![]);
        }

        let ids: Vec<Uuid> = workouts.iter().map(|w| w.id).collect();

        let we_rows = sqlx::query(
            r#"
            SELECT we.id AS we_id,
                   we.workout_id,
                   we.exercise_id,
                   we.position,
                   e.name AS exercise_name
            FROM workout_exercises we
            INNER JOIN exercises e ON e.id = we.exercise_id
            WHERE we.workout_id = ANY($1)
            ORDER BY we.workout_id, we.position
            "#,
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;

        let mut we_ids: Vec<Uuid> = Vec::new();
        let mut we_meta: Vec<(Uuid, Uuid, Uuid, i32, String)> = Vec::new();

        for r in &we_rows {
            let we_id: Uuid = r.try_get("we_id")?;
            let workout_id: Uuid = r.try_get("workout_id")?;
            let exercise_id: Uuid = r.try_get("exercise_id")?;
            let position: i32 = r.try_get("position")?;
            let exercise_name: String = r.try_get("exercise_name")?;
            we_ids.push(we_id);
            we_meta.push((we_id, workout_id, exercise_id, position, exercise_name));
        }

        let mut sets_by_we: std::collections::HashMap<Uuid, Vec<SetSummary>> =
            std::collections::HashMap::new();

        if !we_ids.is_empty() {
            let set_rows = sqlx::query(
                r#"
                SELECT id, workout_exercise_id, set_number, reps, weight_kg, is_warmup, set_type
                FROM sets
                WHERE workout_exercise_id = ANY($1)
                ORDER BY workout_exercise_id, set_number
                "#,
            )
            .bind(&we_ids)
            .fetch_all(&self.pool)
            .await?;

            for r in set_rows {
                let we_id: Uuid = r.try_get("workout_exercise_id")?;
                let summary = SetSummary {
                    id: r.try_get("id")?,
                    set_number: r.try_get("set_number")?,
                    reps: r.try_get("reps")?,
                    weight_kg: r.try_get("weight_kg")?,
                    is_warmup: r.try_get("is_warmup")?,
                    set_type: r.try_get("set_type")?,
                };
                sets_by_we.entry(we_id).or_default().push(summary);
            }
        }

        let mut by_workout: std::collections::HashMap<Uuid, Vec<WorkoutDetailExercise>> =
            std::collections::HashMap::new();

        for (we_id, workout_id, exercise_id, position, exercise_name) in we_meta {
            let sets = sets_by_we.remove(&we_id).unwrap_or_default();
            let entry = WorkoutDetailExercise {
                workout_exercise_id: we_id,
                position,
                exercise: ExerciseSummary {
                    id: exercise_id,
                    name: exercise_name,
                },
                sets,
            };
            by_workout.entry(workout_id).or_default().push(entry);
        }

        let mut out = Vec::with_capacity(workouts.len());
        for w in workouts {
            let exercises = by_workout.remove(&w.id).unwrap_or_default();
            out.push(WorkoutDetail {
                workout: w,
                exercises,
            });
        }

        Ok(out)
    }

    async fn volume_by_exercise(
        &self,
        user_id: Uuid,
        exercise_id: Option<Uuid>,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<ExerciseVolumeRow>> {
        let rows = match (exercise_id, trainee_id) {
            (Some(eid), Some(tid)) => {
                sqlx::query(
                    r#"
                    SELECT
                        e.id AS exercise_id,
                        e.name AS exercise_name,
                        COALESCE(SUM(s.reps::float8 * s.weight_kg), 0.0) AS total_volume
                    FROM exercises e
                    INNER JOIN workout_exercises we ON we.exercise_id = e.id
                    INNER JOIN workouts w ON w.id = we.workout_id
                    LEFT JOIN sets s ON s.workout_exercise_id = we.id
                    WHERE w.user_id = $1 AND w.trainee_id = $2 AND e.id = $3
                    GROUP BY e.id, e.name
                    ORDER BY e.name
                    "#,
                )
                .bind(user_id)
                .bind(tid)
                .bind(eid)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(eid), None) => {
                sqlx::query(
                    r#"
                    SELECT
                        e.id AS exercise_id,
                        e.name AS exercise_name,
                        COALESCE(SUM(s.reps::float8 * s.weight_kg), 0.0) AS total_volume
                    FROM exercises e
                    INNER JOIN workout_exercises we ON we.exercise_id = e.id
                    INNER JOIN workouts w ON w.id = we.workout_id
                    LEFT JOIN sets s ON s.workout_exercise_id = we.id
                    WHERE w.user_id = $1 AND e.id = $2
                    GROUP BY e.id, e.name
                    ORDER BY e.name
                    "#,
                )
                .bind(user_id)
                .bind(eid)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(tid)) => {
                sqlx::query(
                    r#"
                    SELECT
                        e.id AS exercise_id,
                        e.name AS exercise_name,
                        COALESCE(SUM(s.reps::float8 * s.weight_kg), 0.0) AS total_volume
                    FROM exercises e
                    INNER JOIN workout_exercises we ON we.exercise_id = e.id
                    INNER JOIN workouts w ON w.id = we.workout_id
                    LEFT JOIN sets s ON s.workout_exercise_id = we.id
                    WHERE w.user_id = $1 AND w.trainee_id = $2
                    GROUP BY e.id, e.name
                    ORDER BY e.name
                    "#,
                )
                .bind(user_id)
                .bind(tid)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query(
                    r#"
                    SELECT
                        e.id AS exercise_id,
                        e.name AS exercise_name,
                        COALESCE(SUM(s.reps::float8 * s.weight_kg), 0.0) AS total_volume
                    FROM exercises e
                    INNER JOIN workout_exercises we ON we.exercise_id = e.id
                    INNER JOIN workouts w ON w.id = we.workout_id
                    LEFT JOIN sets s ON s.workout_exercise_id = we.id
                    WHERE w.user_id = $1
                    GROUP BY e.id, e.name
                    ORDER BY e.name
                    "#,
                )
                .bind(user_id)
                .fetch_all(&self.pool)
                .await?
            }
        };

        rows.into_iter()
            .map(|r| {
                Ok(ExerciseVolumeRow {
                    exercise_id: r.try_get("exercise_id")?,
                    exercise_name: r.try_get("exercise_name")?,
                    total_volume: r.try_get("total_volume")?,
                })
            })
            .collect()
    }

    async fn personal_records(
        &self,
        user_id: Uuid,
        trainee_id: Option<Uuid>,
    ) -> AppResult<Vec<ExercisePrRow>> {
        let rows = if let Some(tid) = trainee_id {
            sqlx::query(
                r#"
                SELECT
                    e.id AS exercise_id,
                    e.name AS exercise_name,
                    MAX(s.weight_kg) AS max_weight_kg
                FROM exercises e
                INNER JOIN workout_exercises we ON we.exercise_id = e.id
                INNER JOIN workouts w ON w.id = we.workout_id
                INNER JOIN sets s ON s.workout_exercise_id = we.id
                WHERE w.user_id = $1 AND w.trainee_id = $2 AND s.is_warmup = FALSE
                GROUP BY e.id, e.name
                ORDER BY e.name
                "#,
            )
            .bind(user_id)
            .bind(tid)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                r#"
                SELECT
                    e.id AS exercise_id,
                    e.name AS exercise_name,
                    MAX(s.weight_kg) AS max_weight_kg
                FROM exercises e
                INNER JOIN workout_exercises we ON we.exercise_id = e.id
                INNER JOIN workouts w ON w.id = we.workout_id
                INNER JOIN sets s ON s.workout_exercise_id = we.id
                WHERE w.user_id = $1 AND s.is_warmup = FALSE
                GROUP BY e.id, e.name
                ORDER BY e.name
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?
        };

        rows.into_iter()
            .map(|r| {
                Ok(ExercisePrRow {
                    exercise_id: r.try_get("exercise_id")?,
                    exercise_name: r.try_get("exercise_name")?,
                    max_weight_kg: r.try_get("max_weight_kg")?,
                })
            })
            .collect()
    }

    async fn list_recent_sessions_for_trainee(
        &self,
        coach_id: Uuid,
        trainee_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<crate::models::WorkoutSessionSummary>> {
        let rows = sqlx::query_as::<_, crate::models::WorkoutSessionSummary>(
            r#"
            SELECT id, title, started_at
            FROM workouts
            WHERE user_id = $1 AND trainee_id = $2
            ORDER BY started_at DESC
            LIMIT $3
            "#,
        )
        .bind(coach_id)
        .bind(trainee_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    async fn list_exercises(
        &self,
        user_id: Uuid,
        muscle: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> AppResult<Vec<Exercise>> {
        let rows = sqlx::query_as::<_, Exercise>(
            r#"
            SELECT id, user_id, name, muscle, created_at
            FROM exercises
            WHERE user_id = $1
              AND ($2::text IS NULL OR muscle = $2)
              AND ($3::text IS NULL OR name ILIKE '%' || $3 || '%')
            ORDER BY muscle ASC, name ASC
            LIMIT $4
            "#,
        )
        .bind(user_id)
        .bind(muscle)
        .bind(search)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }
}
