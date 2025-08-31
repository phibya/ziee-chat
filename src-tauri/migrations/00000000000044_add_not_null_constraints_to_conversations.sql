-- Add NOT NULL constraints to conversations table to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update conversations table
UPDATE conversations 
SET 
    created_at = COALESCE(created_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE created_at IS NULL OR updated_at IS NULL;

-- Handle active_branch_id - this field should never be NULL in practice
-- but we need to handle it carefully as it references branches table
UPDATE conversations 
SET active_branch_id = (
    SELECT b.id FROM branches b 
    WHERE b.conversation_id = conversations.id 
    LIMIT 1
)
WHERE active_branch_id IS NULL;

-- Now add the NOT NULL constraints
ALTER TABLE conversations
    ALTER COLUMN active_branch_id SET NOT NULL,
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;