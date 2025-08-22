-- Add 'none' to engine_type constraint for remote models
-- This migration updates the check constraints to allow 'none' engine_type for remote models

-- Drop the existing check constraint for engine_type
ALTER TABLE models DROP CONSTRAINT IF EXISTS check_engine_type;

-- Add the updated check constraint that includes 'none'
ALTER TABLE models 
ADD CONSTRAINT check_engine_type 
CHECK (engine_type IN ('mistralrs', 'llamacpp', 'none'));

-- Drop the existing check constraint for engine_settings
ALTER TABLE models DROP CONSTRAINT IF EXISTS check_engine_settings;