-- Add NOT NULL constraints to user group tables to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

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
ALTER TABLE user_group_providers
    ALTER COLUMN assigned_at SET NOT NULL;

ALTER TABLE user_group_rag_providers
    ALTER COLUMN assigned_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;