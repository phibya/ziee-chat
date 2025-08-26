-- Consolidate model engine settings into a single JSON column
-- This migration:
-- 1. Adds a new engine_settings JSONB column
-- 2. Migrates data from separate engine_settings_mistralrs and engine_settings_llamacpp columns
-- 3. Drops the old separate columns

BEGIN;

-- Add the new consolidated engine_settings column
ALTER TABLE models 
  ADD COLUMN engine_settings JSONB DEFAULT '{}';

-- Migrate existing data to the new consolidated structure
UPDATE models 
SET engine_settings = jsonb_build_object(
  'mistralrs', COALESCE(engine_settings_mistralrs, 'null'::jsonb),
  'llamacpp', COALESCE(engine_settings_llamacpp, 'null'::jsonb)
)
WHERE engine_settings_mistralrs IS NOT NULL 
   OR engine_settings_llamacpp IS NOT NULL;

-- Remove null entries to keep the JSON clean
UPDATE models 
SET engine_settings = jsonb_strip_nulls(engine_settings)
WHERE engine_settings IS NOT NULL;

-- Set empty objects to NULL for cleaner storage
UPDATE models 
SET engine_settings = NULL
WHERE engine_settings = '{}'::jsonb;

-- Drop the old separate columns
ALTER TABLE models 
  DROP COLUMN engine_settings_mistralrs,
  DROP COLUMN engine_settings_llamacpp;

COMMIT;