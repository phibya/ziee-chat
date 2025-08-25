-- RAG Core Tables Migration
-- Create RAG providers, instances, and instance files tables

-- Enable Apache AGE extension if not already enabled
-- CREATE EXTENSION IF NOT EXISTS age; -- Disabled for embedded PostgreSQL

-- Create RAG providers table
CREATE TABLE IF NOT EXISTS rag_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL CHECK (provider_type IN ('local', 'chroma', 'weaviate', 'pinecone', 'custom')),
    enabled BOOLEAN DEFAULT FALSE,
    api_key TEXT,
    base_url VARCHAR(512),
    built_in BOOLEAN DEFAULT FALSE,
    proxy_settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create RAG instances table
CREATE TABLE IF NOT EXISTS rag_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES rag_providers(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    alias VARCHAR(255) NOT NULL,
    description TEXT,
    enabled BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT FALSE,
    
    -- Engine configuration (following model pattern)
    engine_type VARCHAR(50) NOT NULL CHECK (engine_type IN ('rag_simple_vector', 'rag_simple_graph')),
    engine_settings_rag_simple_vector JSONB,
    engine_settings_rag_simple_graph JSONB,
    
    -- Model references (using existing models)
    embedding_model_id UUID REFERENCES models(id),
    llm_model_id UUID REFERENCES models(id),
    
    -- Apache AGE graph name (for graph engine instances)
    age_graph_name VARCHAR(255),
    
    -- Parameters storage
    parameters JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(provider_id, alias)
);

-- Create RAG instance files table
CREATE TABLE IF NOT EXISTS rag_instance_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    
    -- RAG-specific fields only
    processing_status VARCHAR(50) DEFAULT 'pending' CHECK (processing_status IN ('pending', 'processing', 'completed', 'failed')),
    processed_at TIMESTAMP WITH TIME ZONE,
    processing_error TEXT,
    rag_metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(rag_instance_id, file_id)
);

-- Create user group RAG providers table
CREATE TABLE IF NOT EXISTS user_group_rag_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES rag_providers(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(group_id, provider_id)
);

-- View to get file info with RAG metadata
CREATE OR REPLACE VIEW rag_instance_files_with_info AS
SELECT 
    rif.*,
    f.filename,
    f.file_size,
    f.mime_type,
    f.checksum,
    f.user_id as file_owner_id,
    f.project_id as file_project_id
FROM rag_instance_files rif
JOIN files f ON f.id = rif.file_id;

-- Create indexes for core tables
CREATE INDEX IF NOT EXISTS idx_rag_instances_provider ON rag_instances(provider_id);
CREATE INDEX IF NOT EXISTS idx_rag_instances_user ON rag_instances(user_id);
CREATE INDEX IF NOT EXISTS idx_rag_instances_project ON rag_instances(project_id);
CREATE INDEX IF NOT EXISTS idx_rag_instances_engine_type ON rag_instances(engine_type);
CREATE INDEX IF NOT EXISTS idx_rag_instances_enabled ON rag_instances(enabled);
CREATE INDEX IF NOT EXISTS idx_rag_instances_active ON rag_instances(is_active);
CREATE INDEX IF NOT EXISTS idx_rag_instances_age_graph ON rag_instances(age_graph_name);
CREATE INDEX IF NOT EXISTS idx_rag_instances_embedding_model ON rag_instances(embedding_model_id);
CREATE INDEX IF NOT EXISTS idx_rag_instances_llm_model ON rag_instances(llm_model_id);

CREATE INDEX IF NOT EXISTS idx_rag_instance_files_instance ON rag_instance_files(rag_instance_id);
CREATE INDEX IF NOT EXISTS idx_rag_instance_files_file ON rag_instance_files(file_id);
CREATE INDEX IF NOT EXISTS idx_rag_instance_files_status ON rag_instance_files(processing_status);

-- Create partial unique index for user_id + alias when user_id is not null
CREATE UNIQUE INDEX IF NOT EXISTS idx_rag_instances_user_alias 
ON rag_instances(user_id, alias) WHERE user_id IS NOT NULL;