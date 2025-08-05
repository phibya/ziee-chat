-- Migration: Drop project_conversations table
-- Since conversations table already has project_id field, we no longer need the junction table

-- Before dropping, migrate any data from project_conversations to conversations.project_id
-- (This is a safety measure in case there's any data that hasn't been migrated)
UPDATE conversations 
SET project_id = pc.project_id
FROM project_conversations pc
WHERE conversations.id = pc.conversation_id 
AND conversations.project_id IS NULL;

-- Drop the indexes first
DROP INDEX IF EXISTS idx_project_conversations_project_id;
DROP INDEX IF EXISTS idx_project_conversations_conversation_id;

-- Drop the table
DROP TABLE IF EXISTS project_conversations;