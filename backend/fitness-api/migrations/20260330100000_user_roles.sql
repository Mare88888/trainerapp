ALTER TABLE users
ADD COLUMN IF NOT EXISTS role VARCHAR(20) NOT NULL DEFAULT 'coach';

UPDATE users
SET role = 'coach'
WHERE role IS NULL OR role = '';
