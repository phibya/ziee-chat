-- Remove checksum columns from models and model_files tables
-- This migration removes checksum functionality to improve model upload performance

-- Remove checksum column from models table
ALTER TABLE models DROP COLUMN IF EXISTS checksum;

-- Remove checksum column from model_files table  
ALTER TABLE model_files DROP COLUMN IF EXISTS checksum;

-- Update comments to reflect removed checksum functionality
COMMENT ON TABLE models IS 'Individual models within each provider (unified table for all model types)';
COMMENT ON TABLE model_files IS 'Individual files that make up models';