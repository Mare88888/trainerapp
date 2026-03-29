-- Trainees managed by a coach (logged-in user)
CREATE TABLE trainees (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    coach_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    display_name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    height_cm DOUBLE PRECISION,
    weight_kg DOUBLE PRECISION,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trainees_coach_id ON trainees (coach_id);

-- Weight / body check-ins for progress
CREATE TABLE trainee_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    trainee_id UUID NOT NULL REFERENCES trainees (id) ON DELETE CASCADE,
    weight_kg DOUBLE PRECISION NOT NULL,
    height_cm DOUBLE PRECISION,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_trainee_metrics_trainee_id ON trainee_metrics (trainee_id);

ALTER TABLE workouts
ADD COLUMN trainee_id UUID REFERENCES trainees (id) ON DELETE SET NULL;

CREATE INDEX idx_workouts_trainee_id ON workouts (trainee_id);
