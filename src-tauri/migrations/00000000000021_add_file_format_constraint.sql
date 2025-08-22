-- Update file_format constraint to use only the three enum values
-- This migration updates existing data, drops the old constraint, and adds a new one with restricted values

-- First, update any existing records that might have invalid formats to 'safetensors' as default
UPDATE models 
SET file_format = 'safetensors' 
WHERE file_format NOT IN ('safetensors', 'pytorch', 'gguf');

-- Drop the existing check constraint
ALTER TABLE models 
DROP CONSTRAINT IF EXISTS check_file_format;

-- Add new constraint with only the three enum values
ALTER TABLE models 
ADD CONSTRAINT check_file_format 
CHECK (file_format IN ('safetensors', 'pytorch', 'gguf'));