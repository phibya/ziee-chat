-- CONSOLIDATED MIGRATION: Complete branching system implementation
-- This migration consolidates all necessary changes for proper branching as specified in CLAUDE.md
-- Run this migration to implement the complete branching system

-- ===============================
-- 1. BRANCHING SYSTEM CORE TABLES
-- ===============================

-- Create branches table for proper branching system
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

-- ========================================
-- 2. ENHANCED MESSAGES TABLE FOR BRANCHING
-- ========================================

-- First, ensure messages table has all required columns from previous migrations
-- Handle parent_id column name change
DO $$
BEGIN
    -- Check if parent_message_id exists and parent_id doesn't
    IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'messages' AND column_name = 'parent_message_id')
       AND NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'messages' AND column_name = 'parent_id') THEN
        ALTER TABLE messages RENAME COLUMN parent_message_id TO parent_id;
    END IF;
END $$;

-- Add columns from previous migrations if they don't exist
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT gen_random_uuid(),
ADD COLUMN IF NOT EXISTS is_active_branch BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS model_provider_id UUID REFERENCES model_providers(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS model_id UUID,
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- Add new branching columns for proper branch system
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS new_branch_id UUID REFERENCES branches(id) ON DELETE CASCADE,
ADD COLUMN IF NOT EXISTS originated_from_id UUID, -- Reference to the original message this was edited from
ADD COLUMN IF NOT EXISTS edit_count INTEGER DEFAULT 0; -- Number of times this message lineage has been edited

-- Ensure alias column exists in model_provider_models
ALTER TABLE model_provider_models 
ADD COLUMN IF NOT EXISTS alias VARCHAR(255);

-- Update existing models with aliases if they don't have them
UPDATE model_provider_models 
SET alias = name 
WHERE alias IS NULL OR alias = '';

-- Make alias required
ALTER TABLE model_provider_models 
ALTER COLUMN alias SET NOT NULL;

-- Add constraint to ensure alias is not empty
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.table_constraints WHERE constraint_name = 'model_provider_models_alias_not_empty') THEN
        ALTER TABLE model_provider_models 
        ADD CONSTRAINT model_provider_models_alias_not_empty 
        CHECK (alias != '');
    END IF;
END $$;

-- ===============================
-- 3. METADATA TABLES
-- ===============================

-- Create message_metadata table for additional information
CREATE TABLE IF NOT EXISTS message_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(message_id, key)
);

-- Create conversation_metadata table
CREATE TABLE IF NOT EXISTS conversation_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(conversation_id, key)
);

-- ===============================
-- 4. INDEXES FOR PERFORMANCE
-- ===============================

-- Indexes for branching system
CREATE INDEX IF NOT EXISTS idx_messages_new_branch_id ON messages(new_branch_id);
CREATE INDEX IF NOT EXISTS idx_messages_originated_from_id ON messages(originated_from_id);
CREATE INDEX IF NOT EXISTS idx_branches_conversation_id ON branches(conversation_id);
CREATE INDEX IF NOT EXISTS idx_conversations_active_branch_id ON conversations(active_branch_id);

-- Indexes from previous migrations
CREATE INDEX IF NOT EXISTS idx_messages_parent_id ON messages(parent_id);
CREATE INDEX IF NOT EXISTS idx_messages_branch_id ON messages(branch_id);
CREATE INDEX IF NOT EXISTS idx_messages_updated_at ON messages(updated_at);
CREATE INDEX IF NOT EXISTS idx_message_metadata_message_id ON message_metadata(message_id);
CREATE INDEX IF NOT EXISTS idx_conversation_metadata_conversation_id ON conversation_metadata(conversation_id);

-- ===============================
-- 5. TRIGGERS AND FUNCTIONS
-- ===============================

-- Function to set default originated_from_id for new messages
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

-- Trigger to automatically set originated_from_id for new messages
DROP TRIGGER IF EXISTS trigger_set_default_originated_from_id ON messages;
CREATE TRIGGER trigger_set_default_originated_from_id
    BEFORE INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION set_default_originated_from_id();

-- Function to update conversation timestamp when messages change
CREATE OR REPLACE FUNCTION update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE conversations 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.conversation_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update conversation timestamp
DROP TRIGGER IF EXISTS update_conversation_on_message ON messages;
CREATE TRIGGER update_conversation_on_message
AFTER INSERT OR UPDATE ON messages
FOR EACH ROW
EXECUTE FUNCTION update_conversation_timestamp();

-- Trigger for messages updated_at (using existing function)
DROP TRIGGER IF EXISTS update_messages_updated_at ON messages;
CREATE TRIGGER update_messages_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ===============================
-- 6. DATA MIGRATION FOR EXISTING DATA
-- ===============================

-- Create main branch for existing conversations that don't have an active branch
DO $$
DECLARE
    conv_record RECORD;
    new_branch_id UUID;
BEGIN
    FOR conv_record IN 
        SELECT id FROM conversations WHERE active_branch_id IS NULL
    LOOP
        -- Create a main branch for this conversation
        INSERT INTO branches (conversation_id, name) 
        VALUES (conv_record.id, 'main') 
        RETURNING id INTO new_branch_id;
        
        -- Set this as the active branch
        UPDATE conversations 
        SET active_branch_id = new_branch_id 
        WHERE id = conv_record.id;
        
        -- Update all messages in this conversation to belong to the main branch
        UPDATE messages 
        SET new_branch_id = new_branch_id 
        WHERE conversation_id = conv_record.id 
        AND new_branch_id IS NULL;
    END LOOP;
END $$;

-- Ensure all messages have edit_count set to 0 if NULL
UPDATE messages 
SET edit_count = 0 
WHERE edit_count IS NULL;

-- Set originated_from_id to message's own ID for messages where it's NULL
UPDATE messages 
SET originated_from_id = id 
WHERE originated_from_id IS NULL;

-- ===============================
-- 7. COMMENTS FOR DOCUMENTATION
-- ===============================

COMMENT ON TABLE branches IS 'Proper branching system table - each branch belongs to a conversation';
COMMENT ON COLUMN branches.name IS 'Optional name for the branch (e.g., "main", "alternative 1")';
COMMENT ON COLUMN conversations.active_branch_id IS 'Currently active branch for this conversation';
COMMENT ON COLUMN messages.new_branch_id IS 'Which branch this message belongs to (new proper branching system)';
COMMENT ON COLUMN messages.originated_from_id IS 'Original message ID this was edited from (for tracking edit lineage)';
COMMENT ON COLUMN messages.edit_count IS 'Number of times this message lineage has been edited';
COMMENT ON COLUMN messages.branch_id IS 'Legacy branch field (will be deprecated)';
COMMENT ON COLUMN messages.is_active_branch IS 'Legacy active branch field (will be deprecated in favor of conversation.active_branch_id)';

-- Migration completed successfully