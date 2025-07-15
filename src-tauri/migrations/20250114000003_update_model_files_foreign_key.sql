-- Migration to fix model_files foreign key constraint
-- This updates the constraint to reference model_provider_models instead of uploaded_models
-- Date: 2025-01-14

-- Drop the existing foreign key constraint
ALTER TABLE model_files DROP CONSTRAINT IF EXISTS model_files_model_id_fkey;

-- Add new foreign key constraint pointing to model_provider_models
ALTER TABLE model_files 
ADD CONSTRAINT model_files_model_id_fkey 
FOREIGN KEY (model_id) REFERENCES model_provider_models(id) ON DELETE CASCADE;

-- Update the comment to reflect the new relationship
COMMENT ON TABLE model_files IS 'Individual files that make up models (now references model_provider_models)';