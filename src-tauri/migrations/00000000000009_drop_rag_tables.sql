-- Drop all RAG-related tables
-- Tables are dropped in dependency order to avoid foreign key constraint errors

-- Drop junction/relationship tables first
DROP TABLE IF EXISTS user_group_rag_providers CASCADE;
DROP TABLE IF EXISTS rag_instance_files CASCADE;

-- Drop RAG instance and processing tables
DROP TABLE IF EXISTS rag_instances CASCADE;
DROP TABLE IF EXISTS rag_processing_pipeline CASCADE;

-- Drop graph and vector storage tables
DROP TABLE IF EXISTS simple_graph_chunks CASCADE;
DROP TABLE IF EXISTS simple_graph_entities CASCADE;
DROP TABLE IF EXISTS simple_graph_relationships CASCADE;
DROP TABLE IF EXISTS simple_vector_documents CASCADE;

-- Drop RAG provider and repository tables
DROP TABLE IF EXISTS rag_providers CASCADE;
DROP TABLE IF EXISTS rag_repositories CASCADE;
