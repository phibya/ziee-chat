-- Make status and error_code columns nullable in rag_instances table
-- Update processing_error column in rag_instance_files table
-- This allows all columns to store NULL values when appropriate
-- Note: No CHECK constraints exist for these enum columns (by design for flexibility)

-- Remove the NOT NULL constraint from status and error_code columns in rag_instances
ALTER TABLE rag_instances 
ALTER COLUMN status DROP NOT NULL,
ALTER COLUMN error_code DROP NOT NULL;

-- Update processing_error column in rag_instance_files table
-- Change from TEXT to VARCHAR and ensure it's nullable
ALTER TABLE rag_instance_files 
ALTER COLUMN processing_error TYPE VARCHAR(100),
ALTER COLUMN processing_error DROP NOT NULL;

-- Update comments to reflect that NULL is allowed
COMMENT ON COLUMN rag_instances.status IS 'Current status of the RAG instance (nullable): indexing, finished, error';

COMMENT ON COLUMN rag_instances.error_code IS 'Error code if status is error (nullable): embedding_model_not_config, embedding_model_not_found, llm_model_not_config, llm_model_not_found, provider_connection_failed, provider_not_found, rag_instance_not_found, indexing_failed, file_processing_failed, database_error, configuration_error';

COMMENT ON COLUMN rag_instance_files.processing_error IS 'Processing error code for file indexing (nullable): text_extraction_failed, unsupported_file_format, file_read_error, chunking_failed, embedding_generation_failed, embedding_model_unavailable, index_storage_failed, content_validation_failed, file_too_large, processing_timeout, processing_error, database_error';