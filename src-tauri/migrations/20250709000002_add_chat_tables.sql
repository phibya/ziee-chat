-- Update existing chat tables to support new branching and metadata structure
-- This migration adds new columns and tables for enhanced chat functionality

-- Add new columns to messages table for enhanced branching
ALTER TABLE messages 
ADD COLUMN IF NOT EXISTS branch_id UUID NOT NULL DEFAULT gen_random_uuid(),
ADD COLUMN IF NOT EXISTS is_active_branch BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS model_provider_id UUID REFERENCES model_providers(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS model_id UUID,
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP;

-- Update messages table to use parent_id instead of parent_message_id for consistency
ALTER TABLE messages RENAME COLUMN parent_message_id TO parent_id;

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

-- Create indexes for new columns and tables
CREATE INDEX IF NOT EXISTS idx_messages_parent_id ON messages(parent_id);
CREATE INDEX IF NOT EXISTS idx_messages_branch_id ON messages(branch_id);
CREATE INDEX IF NOT EXISTS idx_messages_updated_at ON messages(updated_at);
CREATE INDEX IF NOT EXISTS idx_message_metadata_message_id ON message_metadata(message_id);
CREATE INDEX IF NOT EXISTS idx_conversation_metadata_conversation_id ON conversation_metadata(conversation_id);

-- Add trigger to update conversation updated_at when messages are added/updated
CREATE OR REPLACE FUNCTION update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE conversations 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.conversation_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop trigger if exists and recreate
DROP TRIGGER IF EXISTS update_conversation_on_message ON messages;
CREATE TRIGGER update_conversation_on_message
AFTER INSERT OR UPDATE ON messages
FOR EACH ROW
EXECUTE FUNCTION update_conversation_timestamp();

-- Add trigger for messages updated_at
CREATE TRIGGER update_messages_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();