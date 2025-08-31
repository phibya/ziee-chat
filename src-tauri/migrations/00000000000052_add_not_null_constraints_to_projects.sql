-- Add NOT NULL constraints to projects table datetime fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE projects SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE projects SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE projects ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE projects ALTER COLUMN updated_at SET NOT NULL;