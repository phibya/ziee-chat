-- Migration: Drop rag_databases table as we now use rag_instances
-- The rag_databases functionality has been replaced by the new RAG instances system

-- Drop related indexes first
DROP INDEX IF EXISTS idx_rag_databases_provider_id;
DROP INDEX IF EXISTS idx_rag_databases_enabled; 
DROP INDEX IF EXISTS idx_rag_databases_active;
DROP INDEX IF EXISTS idx_rag_databases_provider_alias_unique;

-- Drop the trigger
DROP TRIGGER IF EXISTS update_rag_databases_updated_at ON rag_databases;

-- Drop the table
DROP TABLE IF EXISTS rag_databases;