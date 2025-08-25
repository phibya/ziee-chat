-- Fix RAG Provider Types Migration
-- Update the rag_providers table CHECK constraint to include missing provider types

-- Remove the old CHECK constraint
ALTER TABLE rag_providers DROP CONSTRAINT IF EXISTS rag_providers_provider_type_check;

-- Add the new CHECK constraint with all supported provider types
ALTER TABLE rag_providers 
ADD CONSTRAINT rag_providers_provider_type_check 
CHECK (provider_type IN ('local', 'lightrag', 'ragstack', 'chroma', 'weaviate', 'pinecone', 'custom'));