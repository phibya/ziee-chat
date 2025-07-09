-- Create conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    assistant_id UUID REFERENCES assistants(id) ON DELETE SET NULL,
    model_provider_id UUID REFERENCES model_providers(id) ON DELETE SET NULL,
    model_id UUID,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Index for faster user queries
    INDEX idx_conversations_user_id (user_id),
    INDEX idx_conversations_updated_at (updated_at DESC)
);

-- Create messages table with support for branching
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES messages(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    
    -- Branch information
    branch_id UUID NOT NULL DEFAULT gen_random_uuid(),
    is_active_branch BOOLEAN DEFAULT true,
    
    -- Model information for assistant messages
    model_provider_id UUID REFERENCES model_providers(id) ON DELETE SET NULL,
    model_id UUID,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    -- Indexes
    INDEX idx_messages_conversation_id (conversation_id),
    INDEX idx_messages_parent_id (parent_id),
    INDEX idx_messages_branch_id (branch_id),
    INDEX idx_messages_created_at (created_at)
);

-- Create message_metadata table for additional information
CREATE TABLE message_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(message_id, key),
    INDEX idx_message_metadata_message_id (message_id)
);

-- Create conversation_metadata table
CREATE TABLE conversation_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    
    UNIQUE(conversation_id, key),
    INDEX idx_conversation_metadata_conversation_id (conversation_id)
);

-- Add trigger to update conversation updated_at when messages are added
CREATE OR REPLACE FUNCTION update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE conversations 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.conversation_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_conversation_on_message
AFTER INSERT OR UPDATE ON messages
FOR EACH ROW
EXECUTE FUNCTION update_conversation_timestamp();