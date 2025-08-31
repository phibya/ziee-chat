-- Add NOT NULL constraints to user group tables to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update user_groups table
UPDATE user_groups 
SET 
    is_protected = COALESCE(is_protected, false),
    is_active = COALESCE(is_active, true),
    permissions = COALESCE(permissions, '[]'::jsonb)
WHERE is_protected IS NULL OR is_active IS NULL OR permissions IS NULL;

-- Update user_group_providers table
UPDATE user_group_providers 
SET assigned_at = COALESCE(assigned_at, NOW()) 
WHERE assigned_at IS NULL;

-- Update user_group_rag_providers table  
UPDATE user_group_rag_providers 
SET 
    assigned_at = COALESCE(assigned_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE assigned_at IS NULL OR updated_at IS NULL;

-- Now add the NOT NULL constraints
ALTER TABLE user_groups
    ALTER COLUMN is_protected SET NOT NULL,
    ALTER COLUMN is_active SET NOT NULL,
    ALTER COLUMN permissions SET NOT NULL;

ALTER TABLE user_group_providers
    ALTER COLUMN assigned_at SET NOT NULL;

ALTER TABLE user_group_rag_providers
    ALTER COLUMN assigned_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;