-- RAG Engine-Specific Tables Migration
-- Create engine-specific tables with partitioning for performance

-- Enable vector extension for embedding storage
CREATE EXTENSION IF NOT EXISTS vector;

-- Simple Vector Engine Tables

-- Vector documents table (partitioned by rag_instance_id for scalability)
CREATE TABLE IF NOT EXISTS simple_vector_documents (
    id UUID DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    token_count INTEGER NOT NULL,
    embedding HALFVEC(4000), -- Support up to 4000 dimensions with half precision
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (id, rag_instance_id),
    UNIQUE(rag_instance_id, file_id, chunk_index)
) PARTITION BY HASH (rag_instance_id);

-- Create partitions for simple_vector_documents (16 partitions for balanced distribution)
DO $$
BEGIN
    FOR i IN 0..15 LOOP
        EXECUTE format('CREATE TABLE IF NOT EXISTS simple_vector_documents_%s PARTITION OF simple_vector_documents FOR VALUES WITH (MODULUS 16, REMAINDER %s)', i, i);
    END LOOP;
END
$$;

-- Simple Graph Engine Tables

-- Graph entities table (partitioned by rag_instance_id)
CREATE TABLE IF NOT EXISTS simple_graph_entities (
    id UUID DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    name VARCHAR(512) NOT NULL,
    entity_type VARCHAR(128),
    description TEXT,
    importance_score FLOAT DEFAULT 0.0,
    extraction_metadata JSONB DEFAULT '{}', -- Store extraction confidence, sources, etc.
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (id, rag_instance_id),
    UNIQUE(rag_instance_id, name)
) PARTITION BY HASH (rag_instance_id);

-- Create partitions for simple_graph_entities
DO $$
BEGIN
    FOR i IN 0..15 LOOP
        EXECUTE format('CREATE TABLE IF NOT EXISTS simple_graph_entities_%s PARTITION OF simple_graph_entities FOR VALUES WITH (MODULUS 16, REMAINDER %s)', i, i);
    END LOOP;
END
$$;

-- Graph relationships table (partitioned by rag_instance_id)
CREATE TABLE IF NOT EXISTS simple_graph_relationships (
    id UUID DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    source_entity_id UUID NOT NULL,
    target_entity_id UUID NOT NULL,
    relationship_type VARCHAR(256) NOT NULL,
    description TEXT,
    weight FLOAT DEFAULT 1.0,
    extraction_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (id, rag_instance_id),
    UNIQUE(rag_instance_id, source_entity_id, target_entity_id, relationship_type)
) PARTITION BY HASH (rag_instance_id);

-- Create partitions for simple_graph_relationships
DO $$
BEGIN
    FOR i IN 0..15 LOOP
        EXECUTE format('CREATE TABLE IF NOT EXISTS simple_graph_relationships_%s PARTITION OF simple_graph_relationships FOR VALUES WITH (MODULUS 16, REMAINDER %s)', i, i);
    END LOOP;
END
$$;

-- Graph chunks table - stores text chunks with entity references (partitioned)
CREATE TABLE IF NOT EXISTS simple_graph_chunks (
    id UUID DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    token_count INTEGER NOT NULL,
    entities JSONB DEFAULT '[]', -- Array of entity names found in this chunk
    relationships JSONB DEFAULT '[]', -- Array of relationship descriptions
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (id, rag_instance_id),
    UNIQUE(rag_instance_id, file_id, chunk_index)
) PARTITION BY HASH (rag_instance_id);

-- Create partitions for simple_graph_chunks
DO $$
BEGIN
    FOR i IN 0..15 LOOP
        EXECUTE format('CREATE TABLE IF NOT EXISTS simple_graph_chunks_%s PARTITION OF simple_graph_chunks FOR VALUES WITH (MODULUS 16, REMAINDER %s)', i, i);
    END LOOP;
END
$$;

-- Community detection results table (for graph clustering)
CREATE TABLE IF NOT EXISTS simple_graph_communities (
    id UUID DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    community_id INTEGER NOT NULL,
    entity_ids JSONB NOT NULL, -- Array of entity UUIDs in this community
    summary TEXT, -- Community summary for global searches
    importance_score FLOAT DEFAULT 0.0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (id, rag_instance_id),
    UNIQUE(rag_instance_id, community_id)
) PARTITION BY HASH (rag_instance_id);

-- Create partitions for simple_graph_communities
DO $$
BEGIN
    FOR i IN 0..15 LOOP
        EXECUTE format('CREATE TABLE IF NOT EXISTS simple_graph_communities_%s PARTITION OF simple_graph_communities FOR VALUES WITH (MODULUS 16, REMAINDER %s)', i, i);
    END LOOP;
END
$$;

-- Shared pipeline status table (tracks processing status for files across engines)
CREATE TABLE IF NOT EXISTS rag_processing_pipeline (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rag_instance_id UUID NOT NULL REFERENCES rag_instances(id) ON DELETE CASCADE,
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    pipeline_stage VARCHAR(64) NOT NULL CHECK (pipeline_stage IN (
        'text_extraction', 'chunking', 'embedding', 'entity_extraction', 
        'relationship_extraction', 'indexing', 'completed', 'failed'
    )),
    status VARCHAR(32) NOT NULL CHECK (status IN ('pending', 'processing', 'completed', 'failed')),
    progress_percentage INTEGER DEFAULT 0 CHECK (progress_percentage >= 0 AND progress_percentage <= 100),
    error_message TEXT,
    metadata JSONB DEFAULT '{}', -- Store stage-specific information
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(rag_instance_id, file_id, pipeline_stage)
);

-- Create indexes for performance

-- Simple Vector Engine indexes
CREATE INDEX IF NOT EXISTS idx_simple_vector_docs_instance_file ON simple_vector_documents(rag_instance_id, file_id);
CREATE INDEX IF NOT EXISTS idx_simple_vector_docs_content_hash ON simple_vector_documents(content_hash);
CREATE INDEX IF NOT EXISTS idx_simple_vector_docs_token_count ON simple_vector_documents(token_count);
-- Vector similarity index for embeddings
CREATE INDEX IF NOT EXISTS idx_simple_vector_docs_embedding 
ON simple_vector_documents USING hnsw (embedding halfvec_cosine_ops)
WHERE embedding IS NOT NULL;

-- Simple Graph Engine indexes
CREATE INDEX IF NOT EXISTS idx_simple_graph_entities_instance_name ON simple_graph_entities(rag_instance_id, name);
CREATE INDEX IF NOT EXISTS idx_simple_graph_entities_type ON simple_graph_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_simple_graph_entities_importance ON simple_graph_entities(importance_score DESC);

CREATE INDEX IF NOT EXISTS idx_simple_graph_relationships_source ON simple_graph_relationships(source_entity_id);
CREATE INDEX IF NOT EXISTS idx_simple_graph_relationships_target ON simple_graph_relationships(target_entity_id);
CREATE INDEX IF NOT EXISTS idx_simple_graph_relationships_type ON simple_graph_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_simple_graph_relationships_weight ON simple_graph_relationships(weight DESC);

CREATE INDEX IF NOT EXISTS idx_simple_graph_chunks_instance_file ON simple_graph_chunks(rag_instance_id, file_id);
CREATE INDEX IF NOT EXISTS idx_simple_graph_chunks_entities ON simple_graph_chunks USING GIN(entities);
CREATE INDEX IF NOT EXISTS idx_simple_graph_chunks_content_hash ON simple_graph_chunks(content_hash);

CREATE INDEX IF NOT EXISTS idx_simple_graph_communities_instance ON simple_graph_communities(rag_instance_id);
CREATE INDEX IF NOT EXISTS idx_simple_graph_communities_importance ON simple_graph_communities(importance_score DESC);

-- Pipeline status indexes
CREATE INDEX IF NOT EXISTS idx_rag_pipeline_instance_file ON rag_processing_pipeline(rag_instance_id, file_id);
CREATE INDEX IF NOT EXISTS idx_rag_pipeline_stage_status ON rag_processing_pipeline(pipeline_stage, status);
CREATE INDEX IF NOT EXISTS idx_rag_pipeline_status ON rag_processing_pipeline(status);
CREATE INDEX IF NOT EXISTS idx_rag_pipeline_updated ON rag_processing_pipeline(updated_at);

-- Create materialized views for performance

-- View for entity co-occurrence analysis
CREATE MATERIALIZED VIEW IF NOT EXISTS simple_graph_entity_cooccurrence AS
SELECT 
    sg1.rag_instance_id,
    sg1.entity_id as entity1_id,
    sg2.entity_id as entity2_id,
    COUNT(*) as cooccurrence_count
FROM (
    SELECT 
        rag_instance_id,
        id as chunk_id,
        jsonb_array_elements_text(entities) as entity_id
    FROM simple_graph_chunks
) sg1
JOIN (
    SELECT 
        rag_instance_id,
        id as chunk_id,
        jsonb_array_elements_text(entities) as entity_id
    FROM simple_graph_chunks
) sg2 ON sg1.chunk_id = sg2.chunk_id AND sg1.entity_id < sg2.entity_id
GROUP BY sg1.rag_instance_id, sg1.entity_id, sg2.entity_id;

CREATE INDEX IF NOT EXISTS idx_entity_cooccurrence_instance ON simple_graph_entity_cooccurrence(rag_instance_id);
CREATE INDEX IF NOT EXISTS idx_entity_cooccurrence_count ON simple_graph_entity_cooccurrence(cooccurrence_count DESC);

-- Functions for maintenance

-- Function to refresh materialized views
CREATE OR REPLACE FUNCTION refresh_rag_materialized_views(instance_id UUID DEFAULT NULL)
RETURNS VOID AS $$
BEGIN
    -- Refresh entity co-occurrence view
    IF instance_id IS NOT NULL THEN
        -- For now, refresh entire view (could be optimized for specific instances)
        REFRESH MATERIALIZED VIEW simple_graph_entity_cooccurrence;
    ELSE
        REFRESH MATERIALIZED VIEW simple_graph_entity_cooccurrence;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up orphaned records
CREATE OR REPLACE FUNCTION cleanup_rag_engine_data(instance_id UUID)
RETURNS VOID AS $$
BEGIN
    -- Clean up vector documents
    DELETE FROM simple_vector_documents WHERE rag_instance_id = instance_id;
    
    -- Clean up graph data
    DELETE FROM simple_graph_relationships WHERE rag_instance_id = instance_id;
    DELETE FROM simple_graph_entities WHERE rag_instance_id = instance_id;
    DELETE FROM simple_graph_chunks WHERE rag_instance_id = instance_id;
    DELETE FROM simple_graph_communities WHERE rag_instance_id = instance_id;
    
    -- Clean up pipeline data
    DELETE FROM rag_processing_pipeline WHERE rag_instance_id = instance_id;
    
    -- Refresh materialized views
    PERFORM refresh_rag_materialized_views();
END;
$$ LANGUAGE plpgsql;