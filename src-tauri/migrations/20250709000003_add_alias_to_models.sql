-- Add alias field to model_provider_models table
-- The alias will be used for user-friendly display names
-- The name field will contain the actual provider model ID

ALTER TABLE model_provider_models 
ADD COLUMN IF NOT EXISTS alias VARCHAR NOT NULL DEFAULT '';

-- Update existing models with aliases
-- For now, set alias to the same as name, admins can update them later
UPDATE model_provider_models 
SET alias = name 
WHERE alias = '' OR alias IS NULL;