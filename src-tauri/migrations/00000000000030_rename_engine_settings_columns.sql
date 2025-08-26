-- Rename engine settings columns to remove 'rag_' prefix
-- This migration renames:
-- engine_settings_rag_simple_vector -> engine_settings_simple_vector
-- engine_settings_rag_simple_graph -> engine_settings_simple_graph

BEGIN;

-- Rename the columns
ALTER TABLE rag_instances 
  RENAME COLUMN engine_settings_rag_simple_vector TO engine_settings_simple_vector;

ALTER TABLE rag_instances 
  RENAME COLUMN engine_settings_rag_simple_graph TO engine_settings_simple_graph;

COMMIT;