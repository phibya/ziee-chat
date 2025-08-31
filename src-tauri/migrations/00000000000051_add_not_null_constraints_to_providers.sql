-- Add NOT NULL constraints to providers table fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE providers SET enabled = false WHERE enabled IS NULL;
UPDATE providers SET built_in = false WHERE built_in IS NULL;
UPDATE providers SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE providers SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE providers ALTER COLUMN enabled SET NOT NULL;
ALTER TABLE providers ALTER COLUMN built_in SET NOT NULL;
ALTER TABLE providers ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE providers ALTER COLUMN updated_at SET NOT NULL;