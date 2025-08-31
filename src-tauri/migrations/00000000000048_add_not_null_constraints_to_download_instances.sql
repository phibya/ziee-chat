-- Add NOT NULL constraints to download_instances table datetime fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE download_instances SET started_at = CURRENT_TIMESTAMP WHERE started_at IS NULL;
UPDATE download_instances SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE download_instances SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE download_instances ALTER COLUMN started_at SET NOT NULL;
ALTER TABLE download_instances ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE download_instances ALTER COLUMN updated_at SET NOT NULL;