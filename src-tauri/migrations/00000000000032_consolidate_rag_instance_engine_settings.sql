-- Migration to consolidate RAG instance engine settings from separate columns to single JSON column
-- Add new consolidated engine_settings column
ALTER TABLE rag_instances 
ADD COLUMN engine_settings JSONB DEFAULT '{}' NOT NULL;

-- Migrate existing data from separate columns to consolidated structure
UPDATE rag_instances 
SET engine_settings = jsonb_build_object(
    'simple_vector', COALESCE(engine_settings_simple_vector, 'null'::jsonb),
    'simple_graph', COALESCE(engine_settings_simple_graph, 'null'::jsonb)
)
WHERE engine_settings_simple_vector IS NOT NULL OR engine_settings_simple_graph IS NOT NULL;

-- Set empty object for rows with no engine settings
UPDATE rag_instances 
SET engine_settings = '{}'::jsonb
WHERE engine_settings_simple_vector IS NULL AND engine_settings_simple_graph IS NULL;

-- Drop the old separate columns after data migration
ALTER TABLE rag_instances 
DROP COLUMN engine_settings_simple_vector,
DROP COLUMN engine_settings_simple_graph;