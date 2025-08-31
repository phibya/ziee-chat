-- Add NOT NULL constraints to rag_instances table boolean and datetime fields

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE rag_instances SET enabled = false WHERE enabled IS NULL;
UPDATE rag_instances SET is_active = false WHERE is_active IS NULL;
UPDATE rag_instances SET is_system = false WHERE is_system IS NULL;
UPDATE rag_instances SET created_at = CURRENT_TIMESTAMP WHERE created_at IS NULL;
UPDATE rag_instances SET updated_at = CURRENT_TIMESTAMP WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE rag_instances ALTER COLUMN enabled SET NOT NULL;
ALTER TABLE rag_instances ALTER COLUMN is_active SET NOT NULL;
ALTER TABLE rag_instances ALTER COLUMN is_system SET NOT NULL;
ALTER TABLE rag_instances ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE rag_instances ALTER COLUMN updated_at SET NOT NULL;