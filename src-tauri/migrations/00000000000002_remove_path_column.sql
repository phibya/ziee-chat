-- Remove path column from model_provider_models table
-- Model paths are now determined by the pattern models/{provider_id}/{id}

ALTER TABLE model_provider_models DROP COLUMN IF EXISTS path;