-- Implement proper branching system as specified in CLAUDE.md
-- This migration creates a proper branch-based architecture

-- Create branches table
CREATE TABLE IF NOT EXISTS branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    name VARCHAR(255), -- Optional name for branches (e.g., "main", "alternative 1")
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(conversation_id, name)
);

-- Add active_branch_id to conversations table
ALTER TABLE conversations 
ADD COLUMN IF NOT EXISTS active_branch_id UUID REFERENCES branches(id) ON DELETE SET NULL;

-- Add new columns to messages table for proper branching
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS new_branch_id UUID REFERENCES branches(id) ON DELETE CASCADE,
ADD COLUMN IF NOT EXISTS originated_from_id UUID, -- Reference to the original message this was edited from
ADD COLUMN IF NOT EXISTS edit_count INTEGER DEFAULT 0; -- Number of times this message lineage has been edited

-- Create indexes for the new columns
CREATE INDEX IF NOT EXISTS idx_messages_new_branch_id ON messages(new_branch_id);
CREATE INDEX IF NOT EXISTS idx_messages_originated_from_id ON messages(originated_from_id);
CREATE INDEX IF NOT EXISTS idx_branches_conversation_id ON branches(conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversations_active_branch_id ON conversations(active_branch_id);

-- Add foreign key constraint for originated_from_id
-- Note: This should reference the original message, but we'll allow it to be nullable for now
-- ALTER TABLE messages ADD CONSTRAINT fk_messages_originated_from_id 
-- FOREIGN KEY (originated_from_id) REFERENCES messages(id) ON DELETE SET NULL;

-- Create trigger function to ensure originated_from_id defaults to message id for new messages
CREATE OR REPLACE FUNCTION set_default_originated_from_id()
RETURNS TRIGGER AS $$
BEGIN
    -- If originated_from_id is not set, set it to the message's own ID
    IF NEW.originated_from_id IS NULL THEN
        NEW.originated_from_id = NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically set originated_from_id
DROP TRIGGER IF EXISTS trigger_set_default_originated_from_id ON messages;
CREATE TRIGGER trigger_set_default_originated_from_id
    BEFORE INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION set_default_originated_from_id();