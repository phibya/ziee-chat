-- Remove path column from models table
-- Model paths are now determined by the pattern models/{provider_id}/{id}

ALTER TABLE models DROP COLUMN IF EXISTS path;