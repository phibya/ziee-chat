-- Add project_id column to conversations table
-- Down migrations are not supported, so this is a forward-only migration

ALTER TABLE conversations 
ADD COLUMN project_id UUID REFERENCES projects(id) ON DELETE SET NULL;

-- Create index for faster queries by project_id
CREATE INDEX idx_conversations_project_id ON conversations(project_id);

-- Create composite index for user queries by project
CREATE INDEX idx_conversations_user_project ON conversations(user_id, project_id);