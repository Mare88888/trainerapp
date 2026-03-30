ALTER TABLE sets
ADD COLUMN IF NOT EXISTS set_type TEXT NOT NULL DEFAULT 'normal';

UPDATE sets
SET set_type = CASE WHEN is_warmup THEN 'warmup' ELSE 'normal' END
WHERE set_type IS NULL OR set_type = '';

ALTER TABLE sets
DROP CONSTRAINT IF EXISTS sets_set_type_check;

ALTER TABLE sets
ADD CONSTRAINT sets_set_type_check
CHECK (set_type IN ('warmup', 'normal', 'failure', 'drop'));
