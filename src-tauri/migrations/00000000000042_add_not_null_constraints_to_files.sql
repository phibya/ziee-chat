-- Add NOT NULL constraints to files table to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update files table
UPDATE files 
SET 
    thumbnail_count = COALESCE(thumbnail_count, 0),
    page_count = COALESCE(page_count, 0),
    created_at = COALESCE(created_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE thumbnail_count IS NULL OR page_count IS NULL 
   OR created_at IS NULL OR updated_at IS NULL;

-- Now add the NOT NULL constraints
ALTER TABLE files
    ALTER COLUMN thumbnail_count SET NOT NULL,
    ALTER COLUMN page_count SET NOT NULL,
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;