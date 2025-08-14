-- Add engine columns to models table
-- This migration adds engine_type and engine-specific settings columns

-- Add new columns with defaults
ALTER TABLE models 
ADD COLUMN IF NOT EXISTS engine_type VARCHAR(50) DEFAULT 'mistralrs',
ADD COLUMN IF NOT EXISTS engine_settings_mistralrs JSONB DEFAULT '{}',
ADD COLUMN IF NOT EXISTS engine_settings_llamacpp JSONB DEFAULT '{}';

-- Migrate existing settings data to mistralrs settings
UPDATE models 
SET 
    engine_settings_mistralrs = COALESCE(settings, '{}'),
    engine_type = 'mistralrs'
WHERE settings IS NOT NULL;

-- Update models without settings to have default mistralrs type
UPDATE models 
SET engine_type = 'mistralrs' 
WHERE engine_type IS NULL;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_models_engine_type ON models(engine_type);
CREATE INDEX IF NOT EXISTS idx_models_engine_settings_mistralrs ON models USING gin(engine_settings_mistralrs);
CREATE INDEX IF NOT EXISTS idx_models_engine_settings_llamacpp ON models USING gin(engine_settings_llamacpp);

-- Note: The old 'settings' column should be dropped after verification that the migration worked
-- DROP COLUMN settings; -- Uncomment this line after verifying the migration worked correctly