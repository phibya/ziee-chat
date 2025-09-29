-- Create message_contents table for structured content
CREATE TABLE message_contents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    content_type VARCHAR(50) NOT NULL,
    content JSONB NOT NULL,
    sequence_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_message_contents_message_id ON message_contents(message_id);
CREATE INDEX idx_message_contents_type ON message_contents(content_type);
CREATE INDEX idx_message_contents_sequence ON message_contents(message_id, sequence_order);

-- Migrate existing data from messages.content to structured format
INSERT INTO message_contents (message_id, content_type, content, sequence_order)
SELECT
    id as message_id,
    'text' as content_type,
    jsonb_build_object('text', content) as content,
    0 as sequence_order
FROM messages
WHERE content IS NOT NULL AND content != '';

-- Remove old content column
ALTER TABLE messages DROP COLUMN content;