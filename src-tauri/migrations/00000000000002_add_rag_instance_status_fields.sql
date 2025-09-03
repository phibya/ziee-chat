-- Add status and error_code fields to rag_instances table
-- Note: No constraints are added for enum values as they may change frequently

-- Add status field with default 'none'
ALTER TABLE rag_instances 
ADD COLUMN status VARCHAR(50) NOT NULL DEFAULT 'none';

-- Add error_code field with default 'none'
ALTER TABLE rag_instances 
ADD COLUMN error_code VARCHAR(50) NOT NULL DEFAULT 'none';

-- Add comments for documentation
COMMENT ON COLUMN rag_instances.status IS 'Current status of the RAG instance: none, indexing, finished, error';
COMMENT ON COLUMN rag_instances.error_code IS 'Error code if status is error: none, embedding_model_not_config, embedding_model_not_found, llm_model_not_config, llm_model_not_found, provider_connection_failed, provider_not_found, rag_instance_not_found, indexing_failed, file_processing_failed, database_error, configuration_error';