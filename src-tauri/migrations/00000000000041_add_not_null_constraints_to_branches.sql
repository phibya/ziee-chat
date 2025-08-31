-- Add NOT NULL constraints to branches table to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update branches table
UPDATE branches 
SET created_at = COALESCE(created_at, NOW())
WHERE created_at IS NULL;

-- Now add the NOT NULL constraint
ALTER TABLE branches
    ALTER COLUMN created_at SET NOT NULL;