-- Add NOT NULL constraints to assistants table to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update assistants table
UPDATE assistants 
SET 
    is_template = COALESCE(is_template, false),
    is_default = COALESCE(is_default, false),
    is_active = COALESCE(is_active, true),
    created_at = COALESCE(created_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE is_template IS NULL OR is_default IS NULL OR is_active IS NULL 
   OR created_at IS NULL OR updated_at IS NULL;

-- Now add the NOT NULL constraints
ALTER TABLE assistants
    ALTER COLUMN is_template SET NOT NULL,
    ALTER COLUMN is_default SET NOT NULL,
    ALTER COLUMN is_active SET NOT NULL,
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;