-- Add metadata column to messages table
ALTER TABLE messages ADD COLUMN metadata JSONB DEFAULT NULL;
