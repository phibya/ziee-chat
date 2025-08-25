-- Apache AGE Integration Tables Migration
-- Create tables and functions for Apache AGE graph database integration

-- AGE graph registry table - tracks graph instances
CREATE TABLE IF NOT EXISTS age_graphs (
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
CREATE TABLE IF NOT EXISTS age_query_cache (
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

-- Create indexes for AGE integration
CREATE INDEX IF NOT EXISTS idx_age_graphs_instance ON age_graphs(rag_instance_id);
CREATE INDEX IF NOT EXISTS idx_age_graphs_status ON age_graphs(status);
CREATE INDEX IF NOT EXISTS idx_age_graphs_updated ON age_graphs(last_updated);

CREATE INDEX IF NOT EXISTS idx_age_query_cache_instance ON age_query_cache(rag_instance_id);
CREATE INDEX IF NOT EXISTS idx_age_query_cache_type ON age_query_cache(query_type);
CREATE INDEX IF NOT EXISTS idx_age_query_cache_accessed ON age_query_cache(last_accessed);
CREATE INDEX IF NOT EXISTS idx_age_query_cache_expires ON age_query_cache(expires_at);

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

-- Function to get graph statistics
CREATE OR REPLACE FUNCTION get_age_graph_stats(graph_name_param VARCHAR(255))
    RETURNS TABLE(
                     node_count BIGINT,
                     edge_count BIGINT,
                     label_counts JSONB
                 ) AS $$
DECLARE
result_node_count BIGINT := 0;
    result_edge_count BIGINT := 0;
    result_label_counts JSONB := '{}';
    graph_exists BOOLEAN := FALSE;
BEGIN
    -- Check if graph exists
SELECT EXISTS (
    SELECT 1 FROM ag_graph WHERE name = graph_name_param
) INTO graph_exists;

IF NOT graph_exists THEN
        RETURN QUERY SELECT 0::BIGINT, 0::BIGINT, '{}'::JSONB;
RETURN;
END IF;

    -- Get node count
BEGIN
EXECUTE format('SELECT count(*) FROM cypher(%L, $cypher$ MATCH (n) RETURN n $cypher$) AS (n agtype)', graph_name_param)
    INTO result_node_count;
EXCEPTION WHEN OTHERS THEN
        result_node_count := 0;
END;

    -- Get edge count
BEGIN
EXECUTE format('SELECT count(*) FROM cypher(%L, $cypher$ MATCH ()-[r]->() RETURN r $cypher$) AS (r agtype)', graph_name_param)
    INTO result_edge_count;
EXCEPTION WHEN OTHERS THEN
        result_edge_count := 0;
END;

    -- For now, return basic stats
    -- Label counting would require more complex queries
    result_label_counts := jsonb_build_object('entities', result_node_count, 'relationships', result_edge_count);

RETURN QUERY SELECT result_node_count, result_edge_count, result_label_counts;
END;
$$ LANGUAGE plpgsql;

-- Function to execute Cypher queries safely
CREATE OR REPLACE FUNCTION execute_cypher_query(
    graph_name_param VARCHAR(255),
    cypher_query TEXT,
    max_results INTEGER DEFAULT 1000
)
    RETURNS JSONB AS $$
DECLARE
result JSONB;
    query_with_limit TEXT;
BEGIN
    -- Add LIMIT clause if not present
    IF cypher_query ~* '\bLIMIT\b' THEN
        query_with_limit := cypher_query;
ELSE
        query_with_limit := cypher_query || format(' LIMIT %s', max_results);
END IF;

    -- Execute the query and collect results
BEGIN
EXECUTE format(
        'SELECT jsonb_agg(row_to_json(t)) FROM cypher(%L, %L) AS t(result agtype)',
        graph_name_param,
        query_with_limit
        ) INTO result;

RETURN COALESCE(result, '[]'::JSONB);
EXCEPTION WHEN OTHERS THEN
        -- Return error information
        RETURN jsonb_build_object(
                'error', true,
                'message', SQLERRM,
                'sqlstate', SQLSTATE
               );
END;
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

-- Function to batch insert relationships into AGE graph
CREATE OR REPLACE FUNCTION batch_insert_relationships(
    graph_name_param VARCHAR(255),
    relationships JSONB
)
    RETURNS INTEGER AS $$
DECLARE
relationship JSONB;
    insert_count INTEGER := 0;
    source_name TEXT;
    target_name TEXT;
    rel_type TEXT;
    rel_description TEXT;
    rel_weight FLOAT;
BEGIN
    -- Loop through relationships array
FOR relationship IN SELECT jsonb_array_elements(relationships)
                               LOOP
                        source_name := relationship->>'source_entity';
target_name := relationship->>'target_entity';
            rel_type := COALESCE(relationship->>'relationship_type', 'RELATED_TO');
            rel_description := relationship->>'description';
            rel_weight := COALESCE((relationship->>'weight')::FLOAT, 1.0);

            -- Insert relationship using Cypher
BEGIN
EXECUTE format(
        'SELECT * FROM cypher(%L, $cypher$
         MATCH (s:Entity {name: %L}), (t:Entity {name: %L})
         MERGE (s)-[r:%s {description: %L, weight: %s}]->(t)
         RETURN r
         $cypher$) AS (r agtype)',
        graph_name_param,
        source_name,
        target_name,
        rel_type,
        COALESCE(rel_description, ''),
        rel_weight
        );
insert_count := insert_count + 1;
EXCEPTION WHEN OTHERS THEN
                -- Log error but continue
                RAISE WARNING 'Failed to insert relationship % -> %: %', source_name, target_name, SQLERRM;
END;
END LOOP;

RETURN insert_count;
END;
$$ LANGUAGE plpgsql;

-- Function to find entities by similarity (using name matching for now)
CREATE OR REPLACE FUNCTION find_similar_entities(
    graph_name_param VARCHAR(255),
    query_text TEXT,
    max_results INTEGER DEFAULT 10
)
    RETURNS JSONB AS $$
DECLARE
result JSONB;
BEGIN
BEGIN
EXECUTE format(
        'SELECT jsonb_agg(row_to_json(t)) FROM cypher(%L, $cypher$
         MATCH (e:Entity)
         WHERE e.name ILIKE %L OR e.description ILIKE %L
         RETURN e.name as name, e.type as type, e.description as description, e.importance as importance
         ORDER BY e.importance DESC
         LIMIT %s
         $cypher$) AS t(name agtype, type agtype, description agtype, importance agtype)',
        graph_name_param,
        '%' || query_text || '%',
        '%' || query_text || '%',
        max_results
        ) INTO result;

RETURN COALESCE(result, '[]'::JSONB);
EXCEPTION WHEN OTHERS THEN
        RETURN jsonb_build_object('error', true, 'message', SQLERRM);
END;
END;
$$ LANGUAGE plpgsql;

-- Function to get entity neighborhood
CREATE OR REPLACE FUNCTION get_entity_neighborhood(
    graph_name_param VARCHAR(255),
    entity_name TEXT,
    max_depth INTEGER DEFAULT 2,
    max_results INTEGER DEFAULT 50
)
    RETURNS JSONB AS $$
DECLARE
result JSONB;
    depth_clause TEXT;
BEGIN
    -- Build depth clause
    IF max_depth = 1 THEN
        depth_clause := '-[r]-';
ELSE
        depth_clause := format('-[r*1..%s]-', max_depth);
END IF;

BEGIN
EXECUTE format(
        'SELECT jsonb_agg(row_to_json(t)) FROM cypher(%L, $cypher$
         MATCH (start:Entity {name: %L})%s(end:Entity)
         RETURN DISTINCT end.name as name, end.type as type, end.description as description, end.importance as importance
         ORDER BY end.importance DESC
         LIMIT %s
         $cypher$) AS t(name agtype, type agtype, description agtype, importance agtype)',
        graph_name_param,
        entity_name,
        depth_clause,
        max_results
        ) INTO result;

RETURN COALESCE(result, '[]'::JSONB);
EXCEPTION WHEN OTHERS THEN
        RETURN jsonb_build_object('error', true, 'message', SQLERRM);
END;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up expired cache entries
CREATE OR REPLACE FUNCTION cleanup_age_cache()
    RETURNS INTEGER AS $$
DECLARE
deleted_count INTEGER;
BEGIN
DELETE FROM age_query_cache
WHERE expires_at IS NOT NULL AND expires_at < NOW();

GET DIAGNOSTICS deleted_count = ROW_COUNT;
RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update AGE graph statistics
CREATE OR REPLACE FUNCTION update_age_graph_stats()
    RETURNS TRIGGER AS $$
DECLARE
stats RECORD;
BEGIN
    -- Get updated stats
SELECT * INTO stats FROM get_age_graph_stats(NEW.graph_name);

-- Update the age_graphs table
UPDATE age_graphs
SET
    node_count = stats.node_count,
    edge_count = stats.edge_count,
    last_updated = NOW()
WHERE graph_name = NEW.graph_name;

RETURN NEW;
END;
$$ LANGUAGE plpgsql;