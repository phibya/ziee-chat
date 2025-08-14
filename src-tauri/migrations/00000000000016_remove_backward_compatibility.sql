-- Remove backward compatibility from models table
-- This migration removes the old settings column and makes engine columns required

-- First ensure all models have engine_type set (safety check)
UPDATE models 
SET engine_type = 'mistralrs' 
WHERE engine_type IS NULL OR engine_type = '';

-- Make engine_type NOT NULL and set default
ALTER TABLE models 
ALTER COLUMN engine_type SET NOT NULL,
ALTER COLUMN engine_type SET DEFAULT 'mistralrs';

-- Ensure all models have proper engine settings
UPDATE models 
SET engine_settings_mistralrs = COALESCE(engine_settings_mistralrs, '{}')
WHERE engine_type = 'mistralrs';

UPDATE models 
SET engine_settings_llamacpp = COALESCE(engine_settings_llamacpp, '{}')
WHERE engine_type = 'llamacpp';

-- Now drop the old settings column as it's no longer needed
ALTER TABLE models DROP COLUMN IF EXISTS settings;

-- Add check constraint to ensure valid engine types
ALTER TABLE models 
ADD CONSTRAINT check_engine_type 
CHECK (engine_type IN ('mistralrs', 'llamacpp'));

-- Add check constraint to ensure engine settings are present for the selected engine
ALTER TABLE models 
ADD CONSTRAINT check_engine_settings 
CHECK (
    (engine_type = 'mistralrs' AND engine_settings_mistralrs IS NOT NULL) OR
    (engine_type = 'llamacpp' AND engine_settings_llamacpp IS NOT NULL)
);