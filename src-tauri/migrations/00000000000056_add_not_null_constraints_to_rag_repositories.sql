-- Add NOT NULL constraints to rag_repositories table datetime fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE rag_repositories SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE rag_repositories SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE rag_repositories ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE rag_repositories ALTER COLUMN updated_at SET NOT NULL;