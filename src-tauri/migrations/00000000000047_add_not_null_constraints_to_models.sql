-- Add NOT NULL constraints to models table fields that should not be nullable

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE models SET enabled = false WHERE enabled IS NULL;
UPDATE models SET is_deprecated = false WHERE is_deprecated IS NULL;
UPDATE models SET is_active = false WHERE is_active IS NULL;
UPDATE models SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE models SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE models ALTER COLUMN enabled SET NOT NULL;
ALTER TABLE models ALTER COLUMN is_deprecated SET NOT NULL;
ALTER TABLE models ALTER COLUMN is_active SET NOT NULL;
ALTER TABLE models ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE models ALTER COLUMN updated_at SET NOT NULL;