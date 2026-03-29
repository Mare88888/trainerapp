-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Exercises belong to a user (custom exercise library)
CREATE TABLE exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (user_id, name)
);

-- Workout session
CREATE TABLE workouts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL DEFAULT 'Workout',
    notes TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_workouts_user_id ON workouts (user_id);

-- Exercise performed inside a workout
CREATE TABLE workout_exercises (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    workout_id UUID NOT NULL REFERENCES workouts (id) ON DELETE CASCADE,
    exercise_id UUID NOT NULL REFERENCES exercises (id) ON DELETE CASCADE,
    position INT NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (workout_id, exercise_id)
);

CREATE INDEX idx_workout_exercises_workout_id ON workout_exercises (workout_id);

-- Sets for a workout exercise
CREATE TABLE sets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    workout_exercise_id UUID NOT NULL REFERENCES workout_exercises (id) ON DELETE CASCADE,
    set_number INT NOT NULL,
    reps INT NOT NULL,
    weight_kg DOUBLE PRECISION NOT NULL,
    is_warmup BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (workout_exercise_id, set_number)
);

CREATE INDEX idx_sets_workout_exercise_id ON sets (workout_exercise_id);
