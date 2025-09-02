-- ===============================
-- 1. UTILITY FUNCTIONS
-- ===============================

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
RETURN NEW;
END;
$$ language 'plpgsql';

-- ===============================
-- 2. CORE USER SYSTEM
-- ===============================

-- Create users table (Meteor-like structure with separate tables for arrays)
CREATE TABLE users (
                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                       username VARCHAR(50) NOT NULL UNIQUE,
                       created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                       profile JSONB,
                       is_active BOOLEAN NOT NULL DEFAULT TRUE,
                       is_protected BOOLEAN NOT NULL DEFAULT FALSE,
                       last_login_at TIMESTAMP WITH TIME ZONE,
                       updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create user_emails table (for the emails array)
CREATE TABLE user_emails (
                             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                             user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                             address VARCHAR(255) NOT NULL UNIQUE,
                             verified BOOLEAN NOT NULL DEFAULT FALSE,
                             created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create user_services table (for the services object)
CREATE TABLE user_services (
                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               service_name VARCHAR(50) NOT NULL,
                               service_data JSONB NOT NULL,
                               created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                               UNIQUE(user_id, service_name)
);

-- Create user_login_tokens table (for resume.loginTokens array)
CREATE TABLE user_login_tokens (
                                   id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                   user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                   token VARCHAR(255) NOT NULL UNIQUE,
                                   when_created BIGINT NOT NULL, -- Unix timestamp in milliseconds
                                   expires_at TIMESTAMP WITH TIME ZONE,
                                   created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create user groups table with AWS-style permissions
CREATE TABLE user_groups (
                             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                             name VARCHAR(100) NOT NULL UNIQUE,
                             description TEXT,
                             permissions JSONB DEFAULT '[]' NOT NULL, -- Array format for AWS-style permissions
                             is_protected BOOLEAN DEFAULT FALSE NOT NULL,
                             is_active BOOLEAN DEFAULT TRUE NOT NULL,
                             created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                             updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create user_group_memberships table (many-to-many relationship)
CREATE TABLE user_group_memberships (
                                        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                        user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                                        group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
                                        assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                                        assigned_by UUID REFERENCES users(id),
                                        UNIQUE(user_id, group_id)
);

-- Create user settings table to store personal user preferences
CREATE TABLE user_settings (
                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               key VARCHAR(255) NOT NULL,
                               value JSONB NOT NULL,
                               created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                               updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                               UNIQUE(user_id, key)
);

-- ===============================
-- 3. CONFIGURATION SYSTEM
-- ===============================

-- Create configuration table
CREATE TABLE configurations (
                                id SERIAL PRIMARY KEY,
                                key VARCHAR(255) NOT NULL UNIQUE,
                                value JSONB NOT NULL,
                                description TEXT,
                                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- ===============================
-- 4. MODEL PROVIDER SYSTEM
-- ===============================

-- Create model providers table
CREATE TABLE providers (
                           id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                           name VARCHAR(255) NOT NULL,
                           provider_type VARCHAR(50) NOT NULL CHECK (provider_type IN ('local', 'openai', 'anthropic', 'groq', 'gemini', 'mistral', 'deepseek', 'huggingface', 'custom')),
                           enabled BOOLEAN DEFAULT FALSE NOT NULL,
                           api_key TEXT,
                           base_url VARCHAR(512),
    -- Settings removed - now stored per-model in models.settings JSONB column
                           built_in BOOLEAN DEFAULT FALSE NOT NULL,
                           proxy_settings JSONB DEFAULT '{}',
                           created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                           updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create repositories table for model repositories (Hugging Face, etc.)
CREATE TABLE repositories (
                              id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                              name VARCHAR(255) NOT NULL,
                              url VARCHAR(512) NOT NULL,
                              auth_type VARCHAR(50) NOT NULL CHECK (auth_type IN ('none', 'api_key', 'basic_auth', 'bearer_token')),
                              auth_config JSONB DEFAULT '{}',
                              enabled BOOLEAN DEFAULT TRUE NOT NULL,
                              built_in BOOLEAN DEFAULT FALSE NOT NULL, -- true for built-in repositories like Hugging Face
                              created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                              updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                              UNIQUE(name)
);

-- Create user group model provider relationships
CREATE TABLE user_group_providers (
                                      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                      group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
                                      provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
                                      assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                                      UNIQUE(group_id, provider_id)
);

-- Create model provider models table (unified table for all model types including Candle)
CREATE TABLE models (
                        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                        provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
                        name VARCHAR(255) NOT NULL,
                        alias VARCHAR(255) NOT NULL,
                        description TEXT,
                        enabled BOOLEAN DEFAULT TRUE NOT NULL,
                        is_deprecated BOOLEAN DEFAULT FALSE NOT NULL,
                        is_active BOOLEAN DEFAULT FALSE NOT NULL, -- For local start/stop state
                        capabilities JSONB DEFAULT '{}',
                        parameters JSONB DEFAULT '{}',
    -- Candle-specific fields
                        file_size_bytes BIGINT,
                        validation_status VARCHAR(50) CHECK (validation_status IN (
                                                                                   'pending',        -- Initial status when model is created
                                                                                   'await_upload',   -- For local folder uploads waiting for files
                                                                                   'downloading',    -- For Hugging Face downloads in progress
                                                                                   'processing',     -- While processing uploaded files
                                                                                   'completed',      -- Successfully downloaded/processed
                                                                                   'failed',         -- Download/processing failed
                                                                                   'valid',          -- After successful validation
                                                                                   'invalid',        -- After failed validation
                                                                                   'error',          -- General error state
                                                                                   'validation_warning' -- Downloaded but with validation warnings
                            )),
                        validation_issues JSONB,
    -- Engine-specific settings
                        engine_type VARCHAR(50) NOT NULL DEFAULT 'mistralrs',
                        engine_settings JSONB DEFAULT NULL, -- Consolidated engine settings
                        file_format VARCHAR(20) NOT NULL DEFAULT 'safetensors',
                        source JSONB DEFAULT NULL,
    -- Port number where the model server is running (for local models)
                        port INTEGER,
    -- Process ID of the running model server (for local models)
                        pid INTEGER,
                        created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                        updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                        CONSTRAINT models_alias_not_empty CHECK (alias != ''),
    CONSTRAINT models_provider_id_name_unique UNIQUE (provider_id, name),
    CONSTRAINT check_engine_type CHECK (engine_type IN ('mistralrs', 'llamacpp', 'none')),
    CONSTRAINT check_file_format CHECK (file_format IN ('safetensors', 'pytorch', 'gguf')),
    CONSTRAINT check_source_structure CHECK (
        source IS NULL OR (
            source ? 'type' AND 
            (source->>'type' = 'manual' OR source->>'type' = 'hub')
        )
    )
);

-- ===============================
-- 5. MODEL FILES SYSTEM
-- ===============================

-- Create model_files table for tracking individual files within models
CREATE TABLE model_files (
                             id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                             model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
                             filename VARCHAR(500) NOT NULL,
                             file_path VARCHAR(1000) NOT NULL,
                             file_size_bytes BIGINT NOT NULL,
                             file_type VARCHAR(50) NOT NULL,
                             upload_status VARCHAR(50) DEFAULT 'pending' CHECK (upload_status IN ('pending', 'uploading', 'completed', 'failed')) NOT NULL,
                             uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                             UNIQUE(model_id, filename)
);

-- Create download_instances table for tracking model downloads
CREATE TABLE download_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    request_data JSONB NOT NULL, -- Stores all download parameters
    status VARCHAR(50) NOT NULL CHECK (status IN ('pending', 'downloading', 'completed', 'failed', 'cancelled')),
    progress_data JSONB DEFAULT '{}', -- Stores phase, current, total, message
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    model_id UUID REFERENCES models(id) ON DELETE SET NULL, -- Nullable, filled when download completes
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- ===============================
-- 6. ASSISTANTS SYSTEM
-- ===============================

-- Create assistants table
CREATE TABLE assistants (
                            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                            name VARCHAR(255) NOT NULL,
                            description TEXT,
                            instructions TEXT,
                            parameters JSONB DEFAULT '{}',
                            created_by UUID REFERENCES users(id),
                            is_template BOOLEAN DEFAULT false NOT NULL,
                            is_default BOOLEAN DEFAULT false NOT NULL,
                            is_active BOOLEAN DEFAULT true NOT NULL,
                            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- ===============================
-- 7. PROJECTS SYSTEM
-- ===============================

-- Projects table
CREATE TABLE projects (
                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                          name VARCHAR(255) NOT NULL,
                          description TEXT,
                          instruction TEXT,
                          created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                          updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- ===============================
-- 8. CHAT SYSTEM WITH BRANCHING
-- ===============================

-- Create conversations table
CREATE TABLE conversations (
                               id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                               title VARCHAR(255) NOT NULL,
                               user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                               assistant_id UUID REFERENCES assistants(id),
                               model_id UUID REFERENCES models(id),
                               project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
                               active_branch_id UUID, -- Will be set after branches table is created
                               created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                               updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create branches table for proper branching system
CREATE TABLE branches (
                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                          created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Add foreign key constraint for active_branch_id
ALTER TABLE conversations
    ADD CONSTRAINT fk_conversations_active_branch_id
        FOREIGN KEY (active_branch_id) REFERENCES branches(id) ON DELETE SET NULL;

-- Create messages table without branch relationship
CREATE TABLE messages (
                          id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                          conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                          content TEXT NOT NULL,
                          role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
                          originated_from_id UUID NOT NULL, -- Reference to the original message this was edited from
                          edit_count INTEGER DEFAULT 0 NOT NULL, -- Number of times this message lineage has been edited
                          created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
                          updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Create branch_messages table to manage branch-message relationships
CREATE TABLE branch_messages (
                                 id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                 branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
                                 message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                                 created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                                 is_clone BOOLEAN NOT NULL DEFAULT FALSE,
                                 UNIQUE(branch_id, message_id)
);

-- Create message_metadata table for additional information
CREATE TABLE message_metadata (
                                  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                  message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
                                  key VARCHAR(255) NOT NULL,
                                  value JSONB NOT NULL,
                                  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                                  UNIQUE(message_id, key)
);

-- Create conversation_metadata table
CREATE TABLE conversation_metadata (
                                       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                                       conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                                       key VARCHAR(255) NOT NULL,
                                       value JSONB NOT NULL,
                                       created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                                       UNIQUE(conversation_id, key)
);


-- ===============================
-- 9. FILE MANAGEMENT SYSTEM
-- ===============================

-- Create files table
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    checksum VARCHAR(64),
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    thumbnail_count INTEGER DEFAULT 0 NOT NULL,
    page_count INTEGER DEFAULT 0 NOT NULL,
    processing_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create messages_files table for many-to-many relationship
CREATE TABLE messages_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(message_id, file_id)
);

-- Create provider_files table for provider-specific file mappings
CREATE TABLE provider_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    provider_file_id VARCHAR(255),
    provider_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(file_id, provider_id)
);

-- ===============================
-- 10. RAG SYSTEM
-- ===============================

-- Create RAG providers table
CREATE TABLE rag_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    provider_type VARCHAR NOT NULL, -- 'local', 'lightrag', 'ragstack', 'chroma', 'weaviate', 'pinecone', 'custom'
    enabled BOOLEAN NOT NULL DEFAULT true,
    api_key VARCHAR,
    base_url VARCHAR,
    built_in BOOLEAN NOT NULL DEFAULT false,
    can_user_create_instance BOOLEAN NOT NULL DEFAULT true,
    proxy_settings JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create RAG repositories table
CREATE TABLE rag_repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    url VARCHAR NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    requires_auth BOOLEAN NOT NULL DEFAULT false,
    auth_token VARCHAR,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create RAG instances table
CREATE TABLE rag_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES rag_providers(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    alias VARCHAR(255) NOT NULL,
    description TEXT,
    enabled BOOLEAN DEFAULT TRUE NOT NULL,
    is_active BOOLEAN DEFAULT FALSE NOT NULL,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Engine configuration (following model pattern)
    engine_type VARCHAR(50) NOT NULL CHECK (engine_type IN ('simple_vector', 'simple_graph')),
    engine_settings JSONB DEFAULT '{}' NOT NULL, -- Consolidated engine settings
    
    -- Model references (using existing models)
    embedding_model_id UUID REFERENCES models(id),
    llm_model_id UUID REFERENCES models(id),
    -- Apache AGE graph name (for graph engine instances)
    age_graph_name VARCHAR(255),
    
    -- Parameters storage
    parameters JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    
    UNIQUE(provider_id, alias)
);

-- Create RAG instance files table
CREATE TABLE rag_instance_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    
    -- RAG-specific fields only
    processing_status VARCHAR(50) DEFAULT 'pending' CHECK (processing_status IN ('pending', 'processing', 'completed', 'failed')) NOT NULL,
    processed_at TIMESTAMP WITH TIME ZONE,
    processing_error TEXT,
    rag_metadata JSONB DEFAULT '{}',
    
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    
    UNIQUE(rag_instance_id, file_id)
);

-- Create user group RAG providers table
CREATE TABLE user_group_rag_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES rag_providers(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(group_id, provider_id)
);

-- Simple Vector Engine Tables
CREATE TABLE simple_vector_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    token_count INTEGER NOT NULL,
    embedding TEXT, -- Store as JSON/text for compatibility with embedded PostgreSQL
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rag_instance_id, file_id, chunk_index)
);

-- Simple Graph Engine Tables
CREATE TABLE simple_graph_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    name VARCHAR(512) NOT NULL,
    entity_type VARCHAR(128),
    description TEXT,
    importance_score FLOAT DEFAULT 0.0,
    extraction_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rag_instance_id, name)
);

CREATE TABLE simple_graph_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    source_entity_id UUID NOT NULL,
    target_entity_id UUID NOT NULL,
    relationship_type VARCHAR(256) NOT NULL,
    description TEXT,
    weight FLOAT DEFAULT 1.0,
    extraction_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rag_instance_id, source_entity_id, target_entity_id, relationship_type)
);

CREATE TABLE simple_graph_chunks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    token_count INTEGER NOT NULL,
    entities JSONB DEFAULT '[]',
    relationships JSONB DEFAULT '[]',
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rag_instance_id, file_id, chunk_index)
);

-- Shared pipeline status table
CREATE TABLE rag_processing_pipeline (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    pipeline_stage VARCHAR(64) NOT NULL CHECK (pipeline_stage IN (
        'text_extraction', 'chunking', 'embedding', 'entity_extraction', 
        'relationship_extraction', 'indexing', 'completed', 'failed'
    )),
    status VARCHAR(32) NOT NULL CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(rag_instance_id, file_id, pipeline_stage)
);

-- AGE graph registry table - tracks graph instances
CREATE TABLE age_graphs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    graph_name VARCHAR(255) NOT NULL,
    status VARCHAR(32) DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'error')),
    node_count BIGINT DEFAULT 0,
    edge_count BIGINT DEFAULT 0,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(graph_name)
);

-- AGE query cache table - cache frequently used queries
CREATE TABLE age_query_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    query_hash VARCHAR(64) NOT NULL,
    query_type VARCHAR(32) NOT NULL CHECK (query_type IN ('local', 'global', 'hybrid', 'community')),
    query_params JSONB NOT NULL,
    result_data JSONB NOT NULL,
    hit_count INTEGER DEFAULT 1,
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rag_instance_id, query_hash)
);

-- Function to create a new AGE graph
CREATE OR REPLACE FUNCTION create_age_graph(graph_name_param VARCHAR(255))
RETURNS BOOLEAN AS $$
DECLARE
    graph_exists BOOLEAN := FALSE;
BEGIN
    -- Check if graph already exists
    SELECT EXISTS (
        SELECT 1 FROM ag_graph WHERE name = graph_name_param
    ) INTO graph_exists;
    
    IF NOT graph_exists THEN
        -- Create the graph using Apache AGE
        PERFORM ag_catalog.create_graph(graph_name_param);
        
        -- Add basic label types that we'll use
        BEGIN
            EXECUTE format('SELECT * FROM cypher(%L, $cypher$ CREATE (:Entity {name: ''__init__''}) $cypher$) AS (v agtype);', graph_name_param);
            EXECUTE format('SELECT * FROM cypher(%L, $cypher$ MATCH (n:Entity {name: ''__init__''}) DELETE n $cypher$) AS (v agtype);', graph_name_param);
        EXCEPTION WHEN OTHERS THEN
            -- Ignore initialization errors
            NULL;
        END;
        
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to drop an AGE graph
CREATE OR REPLACE FUNCTION drop_age_graph(graph_name_param VARCHAR(255))
RETURNS BOOLEAN AS $$
DECLARE
    graph_exists BOOLEAN := FALSE;
BEGIN
    -- Check if graph exists
    SELECT EXISTS (
        SELECT 1 FROM ag_graph WHERE name = graph_name_param
    ) INTO graph_exists;
    
    IF graph_exists THEN
        -- Drop the graph using Apache AGE
        PERFORM ag_catalog.drop_graph(graph_name_param, true);
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to batch insert entities into AGE graph
CREATE OR REPLACE FUNCTION batch_insert_entities(
    graph_name_param VARCHAR(255),
    entities JSONB
)
RETURNS INTEGER AS $$
DECLARE
    entity JSONB;
    insert_count INTEGER := 0;
    entity_name TEXT;
    entity_type TEXT;
    entity_description TEXT;
    importance_score FLOAT;
BEGIN
    -- Loop through entities array
    FOR entity IN SELECT jsonb_array_elements(entities)
    LOOP
        entity_name := entity->>'name';
        entity_type := COALESCE(entity->>'entity_type', 'Entity');
        entity_description := entity->>'description';
        importance_score := COALESCE((entity->>'importance_score')::FLOAT, 0.0);
        
        -- Insert entity using Cypher
        BEGIN
            EXECUTE format(
                'SELECT * FROM cypher(%L, $cypher$ MERGE (e:Entity {name: %L, type: %L, description: %L, importance: %s}) RETURN e $cypher$) AS (e agtype)',
                graph_name_param,
                entity_name,
                entity_type,
                COALESCE(entity_description, ''),
                importance_score
            );
            insert_count := insert_count + 1;
        EXCEPTION WHEN OTHERS THEN
            -- Log error but continue
            RAISE WARNING 'Failed to insert entity %: %', entity_name, SQLERRM;
        END;
    END LOOP;
    
    RETURN insert_count;
END;
$$ LANGUAGE plpgsql;

-- ===============================
-- 14. API PROXY SERVER SYSTEM
-- ===============================

-- Create api_proxy_server_models table
CREATE TABLE api_proxy_server_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    alias_id VARCHAR(255) NULL,           -- Human-readable alias for the model
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(model_id)
);

-- Create api_proxy_server_trusted_hosts table
CREATE TABLE api_proxy_server_trusted_hosts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    host VARCHAR(255) NOT NULL,           -- IP address, domain, or CIDR notation
    description TEXT NULL,                -- Optional description of the host
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(host)
);

-- ===============================
-- 15. INDEXES FOR PERFORMANCE
-- ===============================

-- Users and related tables indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_profile ON users USING GIN(profile);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_users_is_protected ON users(is_protected);
CREATE INDEX idx_users_last_login_at ON users(last_login_at);
CREATE INDEX idx_users_updated_at ON users(updated_at);

CREATE INDEX idx_user_emails_user_id ON user_emails(user_id);
CREATE INDEX idx_user_emails_address ON user_emails(address);
CREATE INDEX idx_user_emails_verified ON user_emails(verified);

CREATE INDEX idx_user_services_user_id ON user_services(user_id);
CREATE INDEX idx_user_services_service_name ON user_services(service_name);
CREATE INDEX idx_user_services_data ON user_services USING GIN(service_data);

CREATE INDEX idx_user_login_tokens_user_id ON user_login_tokens(user_id);
CREATE INDEX idx_user_login_tokens_token ON user_login_tokens(token);
CREATE INDEX idx_user_login_tokens_expires_at ON user_login_tokens(expires_at);

CREATE INDEX idx_user_groups_name ON user_groups(name);
CREATE INDEX idx_user_groups_is_protected ON user_groups(is_protected);
CREATE INDEX idx_user_groups_is_active ON user_groups(is_active);
CREATE INDEX idx_user_groups_permissions ON user_groups USING GIN(permissions);

CREATE INDEX idx_user_group_memberships_user_id ON user_group_memberships(user_id);
CREATE INDEX idx_user_group_memberships_group_id ON user_group_memberships(group_id);
CREATE INDEX idx_user_group_memberships_assigned_by ON user_group_memberships(assigned_by);

CREATE INDEX idx_user_group_providers_group_id ON user_group_providers(group_id);
CREATE INDEX idx_user_group_providers_provider_id ON user_group_providers(provider_id);

-- Configuration indexes
CREATE INDEX idx_configurations_key ON configurations(key);
CREATE INDEX idx_configurations_value ON configurations USING GIN(value);

-- User settings indexes
CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
CREATE INDEX idx_user_settings_key ON user_settings(key);
CREATE INDEX idx_user_settings_user_id_key ON user_settings(user_id, key);
CREATE INDEX idx_user_settings_value ON user_settings USING GIN(value);

-- Model provider indexes
CREATE INDEX idx_providers_provider_type ON providers(provider_type);
CREATE INDEX idx_providers_enabled ON providers(enabled);
CREATE INDEX idx_models_provider_id ON models(provider_id);
CREATE INDEX idx_models_enabled ON models(enabled);
CREATE INDEX idx_models_validation_status ON models(validation_status);
CREATE INDEX idx_models_file_size_bytes ON models(file_size_bytes);
CREATE INDEX idx_models_engine_type ON models(engine_type);
CREATE INDEX idx_models_engine_settings ON models USING gin(engine_settings);
CREATE INDEX idx_models_file_format ON models(file_format);
CREATE INDEX idx_models_source_type ON models((source->>'type'));
CREATE INDEX idx_models_source_hub_id ON models((source->>'id')) WHERE source->>'type' = 'hub';

CREATE INDEX idx_model_files_model_id ON model_files(model_id);
CREATE INDEX idx_model_files_upload_status ON model_files(upload_status);

-- Download instances indexes
CREATE INDEX idx_download_instances_provider_id ON download_instances(provider_id);
CREATE INDEX idx_download_instances_repository_id ON download_instances(repository_id);
CREATE INDEX idx_download_instances_status ON download_instances(status);
CREATE INDEX idx_download_instances_started_at ON download_instances(started_at DESC);
CREATE INDEX idx_download_instances_model_id ON download_instances(model_id);

-- Assistant indexes
CREATE INDEX idx_assistants_created_by ON assistants(created_by);
CREATE INDEX idx_assistants_is_template ON assistants(is_template);
CREATE INDEX idx_assistants_is_default ON assistants(is_default);
CREATE INDEX idx_assistants_is_active ON assistants(is_active);
CREATE INDEX idx_assistants_name ON assistants(name);

-- Conversation and branching indexes
CREATE INDEX idx_conversations_user_id ON conversations(user_id);
CREATE INDEX idx_conversations_assistant_id ON conversations(assistant_id);
CREATE INDEX idx_conversations_model_id ON conversations(model_id);
CREATE INDEX idx_conversations_project_id ON conversations(project_id);
CREATE INDEX idx_conversations_user_project ON conversations(user_id, project_id);
CREATE INDEX idx_conversations_active_branch_id ON conversations(active_branch_id);
CREATE INDEX idx_conversations_created_at ON conversations(created_at);

CREATE INDEX idx_branches_conversation_id ON branches(conversation_id);

-- Message indexes
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_originated_from_id ON messages(originated_from_id);
CREATE INDEX idx_messages_updated_at ON messages(updated_at);

-- Branch-message relationship indexes
CREATE INDEX idx_branch_messages_branch_id ON branch_messages(branch_id);
CREATE INDEX idx_branch_messages_message_id ON branch_messages(message_id);
CREATE INDEX idx_branch_messages_created_at ON branch_messages(branch_id, created_at);
CREATE INDEX idx_branch_messages_is_clone ON branch_messages(is_clone);

CREATE INDEX idx_message_metadata_message_id ON message_metadata(message_id);
CREATE INDEX idx_conversation_metadata_conversation_id ON conversation_metadata(conversation_id);

-- Project indexes
CREATE INDEX idx_projects_user_id ON projects(user_id);
CREATE INDEX idx_projects_created_at ON projects(created_at DESC);
CREATE INDEX idx_projects_updated_at ON projects(updated_at DESC);

-- Files indexes
CREATE INDEX idx_files_user_id ON files(user_id);
CREATE INDEX idx_files_project_id ON files(project_id);
CREATE INDEX idx_files_mime_type ON files(mime_type);
CREATE INDEX idx_files_created_at ON files(created_at DESC);
CREATE INDEX idx_files_file_size ON files(file_size);
CREATE INDEX idx_files_checksum ON files(checksum);
CREATE INDEX idx_files_page_count ON files(page_count);
CREATE INDEX idx_files_processing_metadata ON files USING GIN(processing_metadata);

-- Messages-files relationship indexes
CREATE INDEX idx_messages_files_message_id ON messages_files(message_id);
CREATE INDEX idx_messages_files_file_id ON messages_files(file_id);

-- Provider-files relationship indexes
CREATE INDEX idx_provider_files_file_id ON provider_files(file_id);
CREATE INDEX idx_provider_files_provider_id ON provider_files(provider_id);
CREATE INDEX idx_provider_files_provider_file_id ON provider_files(provider_file_id);
CREATE INDEX idx_provider_files_metadata ON provider_files USING GIN(provider_metadata);

-- RAG indexes
CREATE INDEX idx_rag_providers_type ON rag_providers(provider_type);
CREATE INDEX idx_rag_providers_enabled ON rag_providers(enabled);
CREATE INDEX idx_rag_providers_name_unique ON rag_providers(name);

CREATE INDEX idx_rag_repositories_enabled ON rag_repositories(enabled);
CREATE INDEX idx_rag_repositories_priority ON rag_repositories(priority);
CREATE UNIQUE INDEX idx_rag_repositories_url_unique ON rag_repositories(url);

-- RAG instances indexes
CREATE INDEX idx_rag_instances_provider ON rag_instances(provider_id);
CREATE INDEX idx_rag_instances_user ON rag_instances(user_id);
CREATE INDEX idx_rag_instances_project ON rag_instances(project_id);
CREATE INDEX idx_rag_instances_engine_type ON rag_instances(engine_type);
CREATE INDEX idx_rag_instances_enabled ON rag_instances(enabled);
CREATE INDEX idx_rag_instances_active ON rag_instances(is_active);
CREATE INDEX idx_rag_instances_age_graph ON rag_instances(age_graph_name);
CREATE INDEX idx_rag_instances_embedding_model ON rag_instances(embedding_model_id);
CREATE INDEX idx_rag_instances_llm_model ON rag_instances(llm_model_id);
CREATE UNIQUE INDEX idx_rag_instances_user_alias ON rag_instances(user_id, alias) WHERE user_id IS NOT NULL;

CREATE INDEX idx_rag_instance_files_instance ON rag_instance_files(rag_instance_id);
CREATE INDEX idx_rag_instance_files_file ON rag_instance_files(file_id);
CREATE INDEX idx_rag_instance_files_status ON rag_instance_files(processing_status);

CREATE INDEX idx_user_group_rag_providers_group ON user_group_rag_providers(group_id);
CREATE INDEX idx_user_group_rag_providers_provider ON user_group_rag_providers(provider_id);

-- RAG engine indexes
CREATE INDEX idx_simple_vector_docs_instance_file ON simple_vector_documents(rag_instance_id, file_id);
CREATE INDEX idx_simple_vector_docs_content_hash ON simple_vector_documents(content_hash);
CREATE INDEX idx_simple_vector_docs_token_count ON simple_vector_documents(token_count);

CREATE INDEX idx_simple_graph_entities_instance_name ON simple_graph_entities(rag_instance_id, name);
CREATE INDEX idx_simple_graph_entities_type ON simple_graph_entities(entity_type);
CREATE INDEX idx_simple_graph_entities_importance ON simple_graph_entities(importance_score DESC);

CREATE INDEX idx_simple_graph_relationships_source ON simple_graph_relationships(source_entity_id);
CREATE INDEX idx_simple_graph_relationships_target ON simple_graph_relationships(target_entity_id);
CREATE INDEX idx_simple_graph_relationships_type ON simple_graph_relationships(relationship_type);
CREATE INDEX idx_simple_graph_relationships_weight ON simple_graph_relationships(weight DESC);

CREATE INDEX idx_simple_graph_chunks_instance_file ON simple_graph_chunks(rag_instance_id, file_id);
CREATE INDEX idx_simple_graph_chunks_entities ON simple_graph_chunks USING GIN(entities);
CREATE INDEX idx_simple_graph_chunks_content_hash ON simple_graph_chunks(content_hash);

CREATE INDEX idx_rag_pipeline_instance_file ON rag_processing_pipeline(rag_instance_id, file_id);
CREATE INDEX idx_rag_pipeline_stage_status ON rag_processing_pipeline(pipeline_stage, status);
CREATE INDEX idx_rag_pipeline_status ON rag_processing_pipeline(status);
CREATE INDEX idx_rag_pipeline_updated ON rag_processing_pipeline(updated_at);

-- AGE integration indexes
CREATE INDEX idx_age_graphs_instance ON age_graphs(rag_instance_id);
CREATE INDEX idx_age_graphs_status ON age_graphs(status);
CREATE INDEX idx_age_graphs_updated ON age_graphs(last_updated);

CREATE INDEX idx_age_query_cache_instance ON age_query_cache(rag_instance_id);
CREATE INDEX idx_age_query_cache_type ON age_query_cache(query_type);
CREATE INDEX idx_age_query_cache_accessed ON age_query_cache(last_accessed);
CREATE INDEX idx_age_query_cache_expires ON age_query_cache(expires_at);

-- API proxy server indexes
CREATE UNIQUE INDEX idx_api_proxy_server_models_alias 
ON api_proxy_server_models (alias_id) 
WHERE alias_id IS NOT NULL;
CREATE UNIQUE INDEX idx_api_proxy_server_models_default 
ON api_proxy_server_models (is_default) 
WHERE is_default = true;

-- ===============================
-- 16. TRIGGERS AND FUNCTIONS
-- ===============================

-- Function to set default originated_from_id for new messages
CREATE OR REPLACE FUNCTION set_default_originated_from_id()
RETURNS TRIGGER AS $$
BEGIN
    -- If originated_from_id is not set, set it to the message's own ID
    IF NEW.originated_from_id IS NULL THEN
        NEW.originated_from_id = NEW.id;
END IF;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to update conversation timestamp when messages change
CREATE OR REPLACE FUNCTION update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
UPDATE conversations
SET updated_at = CURRENT_TIMESTAMP
WHERE id = NEW.conversation_id;
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to update projects updated_at timestamp
CREATE OR REPLACE FUNCTION update_projects_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at columns
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_groups_updated_at
    BEFORE UPDATE ON user_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_settings_updated_at
    BEFORE UPDATE ON user_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_configurations_updated_at
    BEFORE UPDATE ON configurations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_providers_updated_at
    BEFORE UPDATE ON providers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_repositories_updated_at
    BEFORE UPDATE ON repositories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_models_updated_at
    BEFORE UPDATE ON models
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER download_instances_updated_at 
    BEFORE UPDATE ON download_instances
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_assistants_updated_at
    BEFORE UPDATE ON assistants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger to automatically set originated_from_id for new messages
CREATE TRIGGER trigger_set_default_originated_from_id
    BEFORE INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION set_default_originated_from_id();

-- Trigger to update conversation timestamp when messages change
CREATE TRIGGER update_conversation_on_message
    AFTER INSERT OR UPDATE ON messages
                        FOR EACH ROW
                        EXECUTE FUNCTION update_conversation_timestamp();

-- Trigger for messages updated_at
CREATE TRIGGER update_messages_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Triggers for projects
CREATE TRIGGER update_projects_updated_at_trigger
    BEFORE UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION update_projects_updated_at();

-- Trigger for files updated_at
CREATE TRIGGER update_files_updated_at
    BEFORE UPDATE ON files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- RAG triggers
CREATE TRIGGER update_rag_providers_updated_at
    BEFORE UPDATE ON rag_providers
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rag_repositories_updated_at
    BEFORE UPDATE ON rag_repositories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- API proxy triggers
CREATE TRIGGER update_api_proxy_server_models_updated_at
    BEFORE UPDATE ON api_proxy_server_models
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_api_proxy_server_trusted_hosts_updated_at
    BEFORE UPDATE ON api_proxy_server_trusted_hosts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- RAG triggers
CREATE TRIGGER update_rag_instances_updated_at
    BEFORE UPDATE ON rag_instances
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rag_instance_files_updated_at
    BEFORE UPDATE ON rag_instance_files
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- RAG engine triggers
CREATE TRIGGER update_simple_vector_documents_updated_at
    BEFORE UPDATE ON simple_vector_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_simple_graph_entities_updated_at
    BEFORE UPDATE ON simple_graph_entities
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_simple_graph_relationships_updated_at
    BEFORE UPDATE ON simple_graph_relationships
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_simple_graph_chunks_updated_at
    BEFORE UPDATE ON simple_graph_chunks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rag_processing_pipeline_updated_at
    BEFORE UPDATE ON rag_processing_pipeline
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ===============================
-- 17. DEFAULT DATA
-- ===============================

-- Insert default configuration values
INSERT INTO configurations (key, value, description) VALUES
                                                          ('is_initialized', 'false', 'Indicates whether the application has been initialized'),
                                                          ('enable_user_registration', 'true', 'Controls whether new user registration is enabled'),
                                                          ('default_language', '"en"', 'Default language for the application when user language preference is not set'),
                                                          ('proxy', '{"enabled": false, "url": "", "username": "", "password": "", "no_proxy": "", "ignore_ssl_certificates": false, "proxy_ssl": false, "proxy_host_ssl": false, "peer_ssl": false, "host_ssl": false}', 'Global HTTP proxy configuration'),
                                                          ('api_proxy_server_port', '"8080"', 'API Proxy Server Port'),
                                                          ('api_proxy_server_address', '"127.0.0.1"', 'API Proxy Server Bind Address'),
                                                          ('api_proxy_server_prefix', '"/v1"', 'API Proxy Server URL Prefix'),
                                                          ('api_proxy_server_enabled', 'false', 'Enable/Disable API Proxy Server'),
                                                          ('api_proxy_server_api_key', '""', 'API Key for Proxy Server Authentication'),
                                                          ('api_proxy_server_allow_cors', 'true', 'Enable CORS for API Proxy Server'),
                                                          ('api_proxy_server_log_level', '"info"', 'Log Level: error, warn, info, debug, trace');

-- Create default admin group with wildcard permissions
INSERT INTO user_groups (name, description, permissions, is_protected, is_active)
VALUES (
           'admin',
           'Administrator group with full permissions',
           '["*"]',
           TRUE,
           TRUE
       );

-- Create default user group with basic permissions
INSERT INTO user_groups (name, description, permissions, is_protected, is_active)
VALUES (
           'user',
           'Default user group with basic permissions',
           '["chat::use", "profile::edit", "settings::read", "settings::edit", "settings::delete"]',
           TRUE,
           TRUE
       );

-- Insert default model providers
INSERT INTO providers (name, provider_type, enabled, built_in, base_url) VALUES
                                                                             ('Local', 'local', false, true, null),
                                                                             ('OpenAI', 'openai', false, true, 'https://api.openai.com/v1'),
                                                                             ('Anthropic', 'anthropic', false, true, 'https://api.anthropic.com/v1'),
                                                                             ('Groq', 'groq', false, true, 'https://api.groq.com/openai/v1'),
                                                                             ('Gemini', 'gemini', false, true, 'https://generativelanguage.googleapis.com/v1beta/openai'),
                                                                             ('Mistral', 'mistral', false, true, 'https://api.mistral.ai'),
                                                                             ('DeepSeek', 'deepseek', false, true, 'https://api.deepseek.com/v1'),
                                                                             ('Hugging Face', 'huggingface', false, true, 'https://api-inference.huggingface.co/v1');

-- Insert default repositories (Hugging Face Hub and GitHub)
INSERT INTO repositories (name, url, auth_type, auth_config, enabled, built_in) VALUES
                                                                                    ('Hugging Face Hub', 'https://huggingface.co', 'api_key', '{"api_key": "", "auth_test_api_endpoint": "https://huggingface.co/api/whoami-v2"}', true, true),
                                                                                    ('GitHub', 'https://github.com', 'bearer_token', '{"token": "", "auth_test_api_endpoint": "https://api.github.com/user"}', true, true);

-- Insert default template assistant
INSERT INTO assistants (name, description, instructions, parameters, created_by, is_template, is_default, is_active) VALUES
    ('Default Assistant', 'This is the default assistant.', 'You can use this assistant to chat with the LLM.', '{"stream": true, "temperature": 0.7, "frequency_penalty": 0.7, "presence_penalty": 0.7, "top_p": 0.95, "top_k": 2}', NULL, true, true, true);

-- Insert default trusted hosts for API proxy server
INSERT INTO api_proxy_server_trusted_hosts (host, description, enabled) VALUES
('127.0.0.1', 'Localhost IPv4', true),
('::1', 'Localhost IPv6', true),
('localhost', 'Localhost domain', true);

-- ===============================
-- 18. TABLE COMMENTS
-- ===============================

-- Add comments to document the tables
COMMENT ON TABLE users IS 'Users table with Meteor-like structure';
COMMENT ON TABLE user_groups IS 'User groups with AWS-style permissions in array format';
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults, system settings, and HTTP proxy settings. Document extraction settings have been removed as of migration 12.';
COMMENT ON TABLE user_settings IS 'User settings table for storing personal preferences like appearance, shortcuts, proxy settings, etc.';
COMMENT ON TABLE providers IS 'Model providers table for managing AI model providers like OpenAI, Anthropic, etc.';
COMMENT ON TABLE models IS 'Individual models within each provider (unified table for all model types)';
COMMENT ON TABLE model_files IS 'Individual files that make up models';
COMMENT ON TABLE download_instances IS 'Tracks model download requests and their progress';
COMMENT ON TABLE assistants IS 'Assistants table with template and user-created assistants';
COMMENT ON TABLE conversations IS 'Chat conversations table';
COMMENT ON TABLE messages IS 'Chat messages table without direct branch relationship';
COMMENT ON TABLE branches IS 'Proper branching system table - each branch belongs to a conversation';
COMMENT ON TABLE branch_messages IS 'Many-to-many relationship table between branches and messages with ordering';
COMMENT ON TABLE projects IS 'Projects table for document-based chat contexts';
COMMENT ON TABLE files IS 'Files table for storing uploaded files for projects and chat messages';
COMMENT ON TABLE messages_files IS 'Many-to-many relationship between messages and files';
COMMENT ON TABLE provider_files IS 'Provider-specific file mappings for LLM compatibility';
COMMENT ON TABLE rag_providers IS 'RAG (Retrieval-Augmented Generation) providers table';
COMMENT ON TABLE rag_repositories IS 'RAG repositories table for data source repositories';
COMMENT ON TABLE api_proxy_server_models IS 'API Proxy Server models configuration table';
COMMENT ON TABLE api_proxy_server_trusted_hosts IS 'API Proxy Server trusted hosts whitelist table';
COMMENT ON TABLE rag_instances IS 'RAG instance configurations for document processing';
COMMENT ON TABLE rag_instance_files IS 'Files associated with RAG instances for processing';
COMMENT ON TABLE user_group_rag_providers IS 'User group access to RAG providers';
COMMENT ON TABLE simple_vector_documents IS 'Vector documents for simple vector RAG engine';
COMMENT ON TABLE simple_graph_entities IS 'Graph entities for simple graph RAG engine';
COMMENT ON TABLE simple_graph_relationships IS 'Graph relationships for simple graph RAG engine';
COMMENT ON TABLE simple_graph_chunks IS 'Graph chunks for simple graph RAG engine';
COMMENT ON TABLE rag_processing_pipeline IS 'RAG processing pipeline status tracking';
COMMENT ON TABLE age_graphs IS 'Apache AGE graph registry table - tracks graph instances';
COMMENT ON TABLE age_query_cache IS 'Apache AGE query cache table - cache frequently used queries';

-- Column comments
COMMENT ON COLUMN user_groups.permissions IS 'AWS-style permissions stored as JSON array. Supports wildcards like "users::*", "groups::*", and "*"';
COMMENT ON COLUMN user_settings.key IS 'Setting key using camelCase format (e.g., "appearance.theme", "appearance.fontSize")';
COMMENT ON COLUMN user_settings.value IS 'Setting value stored as JSONB for flexibility';
COMMENT ON COLUMN providers.provider_type IS 'Type of provider: local, openai, anthropic, groq, gemini, mistral, custom';
COMMENT ON COLUMN models.name IS 'Unique model identifier within a provider';
COMMENT ON COLUMN models.alias IS 'Human-readable display name (can be duplicated across providers)';
COMMENT ON COLUMN models.is_active IS 'Whether the model is currently running (for local models)';
COMMENT ON COLUMN models.file_size_bytes IS 'Total size of all model files in bytes - for Candle models only';
COMMENT ON COLUMN models.validation_status IS 'Status of model validation and processing - for Candle models only';
COMMENT ON COLUMN models.validation_issues IS 'JSON array of validation issues if any - for Candle models only';
COMMENT ON COLUMN providers.proxy_settings IS 'JSON object containing all proxy configuration settings including enabled, url, username, password, no_proxy, SSL settings, etc.';
COMMENT ON COLUMN model_files.filename IS 'Original filename of the uploaded file';
COMMENT ON COLUMN model_files.file_path IS 'Storage path of the file in the filesystem';
COMMENT ON COLUMN model_files.file_type IS 'Type of file (e.g., model, tokenizer, config, safetensors)';
COMMENT ON COLUMN model_files.upload_status IS 'Upload status: pending, uploading, completed, failed';
COMMENT ON COLUMN download_instances.request_data IS 'JSON containing all download parameters like model_id, quantization, etc.';
COMMENT ON COLUMN download_instances.progress_data IS 'JSON with structure: {phase: string, current: number, total: number, message: string}';
COMMENT ON COLUMN download_instances.model_id IS 'References the created model entry after successful download';
COMMENT ON TABLE models IS 'Individual models within each provider (unified table for all model types)';
COMMENT ON TABLE model_files IS 'Individual files that make up models';
COMMENT ON TABLE providers IS 'Model providers (connection/authentication only, settings moved to individual models)';
COMMENT ON COLUMN models.port IS 'Port number where the model server is running (for Candle models only)';
COMMENT ON COLUMN models.file_format IS 'Model file format: safetensors, gguf, bin, pt, pth, onnx';
COMMENT ON COLUMN models.source IS 'Source information: {type: "manual"|"hub", id: string|null}';
COMMENT ON COLUMN assistants.is_template IS 'Whether this assistant is a template (admin-created) that can be cloned by users';
COMMENT ON COLUMN assistants.is_default IS 'Whether this template assistant is automatically cloned for new users';
COMMENT ON COLUMN assistants.created_by IS 'User who created this assistant (NULL for system/template assistants)';
COMMENT ON COLUMN conversations.assistant_id IS 'Assistant used in this conversation';
COMMENT ON COLUMN conversations.model_id IS 'Specific model used in this conversation';
COMMENT ON COLUMN conversations.active_branch_id IS 'Currently active branch for this conversation';
COMMENT ON COLUMN messages.originated_from_id IS 'Original message ID this was edited from (for tracking edit lineage)';
COMMENT ON COLUMN messages.edit_count IS 'Number of times this message lineage has been edited';
COMMENT ON COLUMN messages.role IS 'Message role: user, assistant, or system';
COMMENT ON COLUMN branch_messages.branch_id IS 'Reference to the branch containing this message';
COMMENT ON COLUMN branch_messages.message_id IS 'Reference to the message in this branch';
COMMENT ON COLUMN branch_messages.is_clone IS 'Indicates whether this message is a clone (belongs to multiple branches) or is unique to this branch';
COMMENT ON COLUMN files.filename IS 'Original filename with extension';
COMMENT ON COLUMN files.file_size IS 'File size in bytes';
COMMENT ON COLUMN files.mime_type IS 'MIME type of the file';
COMMENT ON COLUMN files.checksum IS 'SHA-256 hash of the file for integrity verification';
COMMENT ON COLUMN files.project_id IS 'Project this file belongs to (nullable for general uploads)';
COMMENT ON COLUMN files.thumbnail_count IS 'Number of thumbnails generated for this file';
COMMENT ON COLUMN files.page_count IS 'Number of high-quality images/pages generated for this file';
COMMENT ON COLUMN files.processing_metadata IS 'Processing results and metadata (text length, pages, dimensions, etc.)';
COMMENT ON COLUMN provider_files.provider_file_id IS 'Provider-specific file ID if applicable';
COMMENT ON COLUMN provider_files.provider_metadata IS 'Provider-specific metadata and processing info';

-- Constraint comments
COMMENT ON CONSTRAINT models_provider_id_name_unique ON models IS 'Ensures model IDs (name) are unique per provider, while allowing duplicate display names (alias) across providers';