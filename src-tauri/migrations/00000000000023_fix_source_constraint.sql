-- Fix source structure constraint to not require 'id' field
-- The 'id' field should be optional for manual downloads

-- Drop the existing check constraint
ALTER TABLE models DROP CONSTRAINT IF EXISTS check_source_structure;