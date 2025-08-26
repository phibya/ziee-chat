-- Add can_user_create_instance column to rag_providers table
-- This field controls whether users can create instances with this RAG provider

ALTER TABLE rag_providers 
ADD COLUMN can_user_create_instance BOOLEAN NOT NULL DEFAULT true;