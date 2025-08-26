-- Remove can_create_instance column from user_group_rag_providers table
-- This field is being removed to simplify RAG provider assignments to user groups

ALTER TABLE user_group_rag_providers 
DROP COLUMN IF EXISTS can_create_instance;