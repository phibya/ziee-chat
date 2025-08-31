-- Add NOT NULL constraints to repositories table datetime and boolean fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE repositories SET enabled = true WHERE enabled IS NULL;
UPDATE repositories SET built_in = false WHERE built_in IS NULL;
UPDATE repositories SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE repositories SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE repositories ALTER COLUMN enabled SET NOT NULL;
ALTER TABLE repositories ALTER COLUMN built_in SET NOT NULL;
ALTER TABLE repositories ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE repositories ALTER COLUMN updated_at SET NOT NULL;