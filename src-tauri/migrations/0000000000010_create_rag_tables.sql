-- Migration to create RAG (Retrieval-Augmented Generation) tables
-- This migration creates tables for RAG providers, databases, and repositories

-- Create RAG providers table
CREATE TABLE IF NOT EXISTS rag_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    provider_type VARCHAR NOT NULL, -- 'local', 'lightrag', 'ragstack', 'chroma', 'weaviate', 'pinecone', 'custom'
    enabled BOOLEAN NOT NULL DEFAULT true,
    api_key VARCHAR,
    base_url VARCHAR,
    built_in BOOLEAN NOT NULL DEFAULT false,
    proxy_settings JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create index on provider_type for performance
CREATE INDEX IF NOT EXISTS idx_rag_providers_type ON rag_providers(provider_type);
CREATE INDEX IF NOT EXISTS idx_rag_providers_enabled ON rag_providers(enabled);

-- Create RAG databases table
CREATE TABLE IF NOT EXISTS rag_databases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES rag_providers(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    alias VARCHAR NOT NULL,
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_active BOOLEAN NOT NULL DEFAULT false, -- Only meaningful for local providers
    collection_name VARCHAR,
    embedding_model VARCHAR,
    chunk_size INTEGER NOT NULL DEFAULT 1000,
    chunk_overlap INTEGER NOT NULL DEFAULT 200,
    capabilities JSONB DEFAULT '{}', -- semantic_search, hybrid_search, metadata_filtering, similarity_threshold
    settings JSONB DEFAULT '{}', -- provider-specific settings
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_rag_databases_provider_id ON rag_databases(provider_id);
CREATE INDEX IF NOT EXISTS idx_rag_databases_enabled ON rag_databases(enabled);
CREATE INDEX IF NOT EXISTS idx_rag_databases_active ON rag_databases(is_active);

-- Create RAG repositories table
CREATE TABLE IF NOT EXISTS rag_repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    description TEXT,
    url VARCHAR NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    requires_auth BOOLEAN NOT NULL DEFAULT false,
    auth_token VARCHAR,
    priority INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_rag_repositories_enabled ON rag_repositories(enabled);
CREATE INDEX IF NOT EXISTS idx_rag_repositories_priority ON rag_repositories(priority);

-- Create unique constraints to prevent duplicates
CREATE UNIQUE INDEX IF NOT EXISTS idx_rag_providers_name_unique ON rag_providers(name);
CREATE UNIQUE INDEX IF NOT EXISTS idx_rag_databases_provider_alias_unique ON rag_databases(provider_id, alias);
CREATE UNIQUE INDEX IF NOT EXISTS idx_rag_repositories_url_unique ON rag_repositories(url);

-- Create trigger function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_rag_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for updated_at timestamps
CREATE TRIGGER update_rag_providers_updated_at
    BEFORE UPDATE ON rag_providers
    FOR EACH ROW
    EXECUTE FUNCTION update_rag_updated_at();

CREATE TRIGGER update_rag_databases_updated_at
    BEFORE UPDATE ON rag_databases
    FOR EACH ROW
    EXECUTE FUNCTION update_rag_updated_at();

CREATE TRIGGER update_rag_repositories_updated_at
    BEFORE UPDATE ON rag_repositories
    FOR EACH ROW
    EXECUTE FUNCTION update_rag_updated_at();