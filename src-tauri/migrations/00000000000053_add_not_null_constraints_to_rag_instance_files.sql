-- Add NOT NULL constraints to rag_instance_files table datetime fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE rag_instance_files SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE rag_instance_files SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE rag_instance_files ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE rag_instance_files ALTER COLUMN updated_at SET NOT NULL;