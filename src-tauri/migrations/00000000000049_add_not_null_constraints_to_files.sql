-- Add NOT NULL constraints to files table datetime fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE files SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE files SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE files ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE files ALTER COLUMN updated_at SET NOT NULL;

-- Provider files table 
UPDATE provider_files SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
ALTER TABLE provider_files ALTER COLUMN created_at SET NOT NULL;

-- Message files table
UPDATE messages_files SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
ALTER TABLE messages_files ALTER COLUMN created_at SET NOT NULL;