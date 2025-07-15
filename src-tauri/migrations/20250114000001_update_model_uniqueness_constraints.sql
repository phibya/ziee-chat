-- Migration to update model uniqueness constraints
-- Allow duplicate display names (alias) across providers
-- Make model ID (name) + provider_id combination unique instead
-- Date: 2025-01-14

-- Remove the existing unique constraint on (provider_id, alias) for uploaded_models
ALTER TABLE uploaded_models DROP CONSTRAINT IF EXISTS uploaded_models_provider_id_alias_key;

-- Add unique constraint on (provider_id, name) for uploaded_models
-- This ensures model IDs are unique per provider, but display names can be duplicated
ALTER TABLE uploaded_models ADD CONSTRAINT uploaded_models_provider_id_name_unique UNIQUE (provider_id, name);

-- Also update model_provider_models table to follow the same pattern
-- Remove any existing unique constraints on alias or name
ALTER TABLE model_provider_models DROP CONSTRAINT IF EXISTS model_provider_models_provider_id_alias_key;
ALTER TABLE model_provider_models DROP CONSTRAINT IF EXISTS model_provider_models_alias_key;
ALTER TABLE model_provider_models DROP CONSTRAINT IF EXISTS model_provider_models_name_key;

-- Add unique constraint on (provider_id, name) for model_provider_models
-- This ensures model IDs are unique per provider, but display names can be duplicated
ALTER TABLE model_provider_models ADD CONSTRAINT model_provider_models_provider_id_name_unique UNIQUE (provider_id, name);

-- Update table comments to reflect the new constraint logic
COMMENT ON CONSTRAINT uploaded_models_provider_id_name_unique ON uploaded_models IS 'Ensures model IDs (name) are unique per provider, while allowing duplicate display names (alias) across providers';
COMMENT ON CONSTRAINT model_provider_models_provider_id_name_unique ON model_provider_models IS 'Ensures model IDs (name) are unique per provider, while allowing duplicate display names (alias) across providers';

-- Update column comments to clarify the difference between name and alias
COMMENT ON COLUMN uploaded_models.name IS 'Unique model identifier within a provider (auto-generated for Candle models)';
COMMENT ON COLUMN uploaded_models.alias IS 'Human-readable display name (can be duplicated across providers)';
COMMENT ON COLUMN model_provider_models.name IS 'Unique model identifier within a provider';
COMMENT ON COLUMN model_provider_models.alias IS 'Human-readable display name (can be duplicated across providers)';