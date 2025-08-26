-- Migration to update RAG engine type constraint from 'rag_simple_vector'/'rag_simple_graph' to 'simple_vector'/'simple_graph'

-- First update existing data to use new values
UPDATE rag_instances 
SET engine_type = 'simple_vector' 
WHERE engine_type = 'rag_simple_vector';

UPDATE rag_instances 
SET engine_type = 'simple_graph' 
WHERE engine_type = 'rag_simple_graph';

-- Drop the old constraint
ALTER TABLE rag_instances 
DROP CONSTRAINT IF EXISTS rag_instances_engine_type_check;

-- Add new constraint with updated values
ALTER TABLE rag_instances 
ADD CONSTRAINT rag_instances_engine_type_check 
CHECK (engine_type IN ('simple_vector', 'simple_graph'));