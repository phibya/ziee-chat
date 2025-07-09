-- Make alias field required for all models
-- First ensure all existing models have an alias
UPDATE model_provider_models 
SET alias = name 
WHERE alias IS NULL OR alias = '';

-- Now alter the column to be NOT NULL
ALTER TABLE model_provider_models 
ALTER COLUMN alias SET NOT NULL;

-- Add a check constraint to ensure alias is not empty
ALTER TABLE model_provider_models 
ADD CONSTRAINT model_provider_models_alias_not_empty 
CHECK (alias != '');