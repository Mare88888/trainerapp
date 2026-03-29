ALTER TABLE exercises
ADD COLUMN IF NOT EXISTS muscle TEXT NOT NULL DEFAULT 'general';

WITH library(name, muscle) AS (
    VALUES
        -- Chest
        ('Barbell Bench Press', 'chest'),
        ('Incline Barbell Bench Press', 'chest'),
        ('Decline Barbell Bench Press', 'chest'),
        ('Dumbbell Bench Press', 'chest'),
        ('Incline Dumbbell Press', 'chest'),
        ('Decline Dumbbell Press', 'chest'),
        ('Chest Fly (Dumbbell)', 'chest'),
        ('Incline Chest Fly', 'chest'),
        ('Decline Chest Fly', 'chest'),
        ('Machine Chest Press', 'chest'),
        ('Pec Deck Fly', 'chest'),
        ('Cable Chest Fly', 'chest'),
        ('Low-to-High Cable Fly', 'chest'),
        ('High-to-Low Cable Fly', 'chest'),
        ('Push-Up', 'chest'),
        ('Incline Push-Up', 'chest'),
        ('Decline Push-Up', 'chest'),
        ('Weighted Push-Up', 'chest'),
        ('Chest Dip', 'chest'),

        -- Back
        ('Deadlift', 'back'),
        ('Romanian Deadlift', 'hamstrings'),
        ('Rack Pull', 'back'),
        ('Pull-Up', 'back'),
        ('Chin-Up', 'back'),
        ('Neutral Grip Pull-Up', 'back'),
        ('Weighted Pull-Up', 'back'),
        ('Lat Pulldown', 'back'),
        ('Wide Grip Lat Pulldown', 'back'),
        ('Close Grip Lat Pulldown', 'back'),
        ('Seated Cable Row', 'back'),
        ('Bent Over Barbell Row', 'back'),
        ('Pendlay Row', 'back'),
        ('T-Bar Row', 'back'),
        ('Single Arm Dumbbell Row', 'back'),
        ('Machine Row', 'back'),
        ('Chest Supported Row', 'back'),
        ('Inverted Row', 'back'),
        ('Straight Arm Pulldown', 'back'),

        -- Shoulders
        ('Overhead Press (Barbell)', 'shoulders'),
        ('Seated Barbell Press', 'shoulders'),
        ('Dumbbell Shoulder Press', 'shoulders'),
        ('Arnold Press', 'shoulders'),
        ('Lateral Raise', 'shoulders'),
        ('Front Raise', 'shoulders'),
        ('Rear Delt Fly', 'shoulders'),
        ('Reverse Pec Deck', 'shoulders'),
        ('Cable Lateral Raise', 'shoulders'),
        ('Cable Front Raise', 'shoulders'),
        ('Upright Row', 'shoulders'),
        ('Face Pull', 'shoulders'),
        ('Handstand Push-Up', 'shoulders'),

        -- Arms / Biceps
        ('Barbell Curl', 'biceps'),
        ('EZ Bar Curl', 'biceps'),
        ('Dumbbell Curl', 'biceps'),
        ('Alternating Dumbbell Curl', 'biceps'),
        ('Hammer Curl', 'biceps'),
        ('Incline Dumbbell Curl', 'biceps'),
        ('Preacher Curl', 'biceps'),
        ('Concentration Curl', 'biceps'),
        ('Cable Curl', 'biceps'),
        ('Spider Curl', 'biceps'),
        ('Reverse Curl', 'biceps'),

        -- Arms / Triceps
        ('Close Grip Bench Press', 'triceps'),
        ('Triceps Pushdown', 'triceps'),
        ('Rope Pushdown', 'triceps'),
        ('Overhead Triceps Extension', 'triceps'),
        ('Skullcrusher', 'triceps'),
        ('Dumbbell Skullcrusher', 'triceps'),
        ('Triceps Kickback', 'triceps'),
        ('Bench Dip', 'triceps'),
        ('Weighted Dip', 'triceps'),

        -- Legs / Quads
        ('Barbell Squat', 'quads'),
        ('Front Squat', 'quads'),
        ('Hack Squat', 'quads'),
        ('Leg Press', 'quads'),
        ('Bulgarian Split Squat', 'quads'),
        ('Lunges', 'quads'),
        ('Walking Lunges', 'quads'),
        ('Step-Up', 'quads'),
        ('Leg Extension', 'quads'),

        -- Legs / Hamstrings + Glutes
        ('Stiff Leg Deadlift', 'hamstrings'),
        ('Hip Thrust', 'glutes'),
        ('Glute Bridge', 'glutes'),
        ('Good Morning', 'hamstrings'),
        ('Nordic Curl', 'hamstrings'),
        ('Leg Curl (Seated)', 'hamstrings'),
        ('Leg Curl (Lying)', 'hamstrings'),

        -- Legs / Calves
        ('Standing Calf Raise', 'calves'),
        ('Seated Calf Raise', 'calves'),
        ('Calf Press', 'calves'),

        -- Core
        ('Crunch', 'core'),
        ('Sit-Up', 'core'),
        ('Hanging Leg Raise', 'core'),
        ('Lying Leg Raise', 'core'),
        ('Cable Crunch', 'core'),
        ('Russian Twist', 'core'),
        ('Bicycle Crunch', 'core'),
        ('Plank', 'core'),
        ('Side Plank', 'core'),
        ('Ab Wheel Rollout', 'core'),
        ('Mountain Climbers', 'core'),

        -- Full Body / Olympic
        ('Clean', 'full body'),
        ('Power Clean', 'full body'),
        ('Hang Clean', 'full body'),
        ('Snatch', 'full body'),
        ('Power Snatch', 'full body'),
        ('Clean and Jerk', 'full body'),
        ('Thruster', 'full body'),
        ('Farmer''s Walk', 'full body'),

        -- Bodyweight / Advanced
        ('Muscle-Up', 'bodyweight'),
        ('Front Lever', 'bodyweight'),
        ('Back Lever', 'bodyweight'),
        ('Planche', 'bodyweight'),
        ('L-Sit', 'bodyweight'),
        ('Pistol Squat', 'bodyweight'),
        ('Dragon Flag', 'bodyweight'),
        ('Human Flag', 'bodyweight')
)
INSERT INTO exercises (user_id, name, muscle)
SELECT u.id, l.name, l.muscle
FROM users u
CROSS JOIN library l
ON CONFLICT (user_id, name)
DO UPDATE SET muscle = EXCLUDED.muscle;
