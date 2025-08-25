-- RAG Instance Manager Enhancement Migration
-- Add can_create_instance permission to user_group_rag_providers and is_system to rag_instances

-- Add can_create_instance permission to existing table
ALTER TABLE user_group_rag_providers 
ADD COLUMN IF NOT EXISTS can_create_instance BOOLEAN DEFAULT false;

-- Add updated_at timestamp
ALTER TABLE user_group_rag_providers 
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Create index for can_create_instance permission lookups
CREATE INDEX IF NOT EXISTS idx_user_group_rag_providers_create_instance 
ON user_group_rag_providers(group_id, provider_id) WHERE can_create_instance = true;

-- Add is_system field for system vs user instances
ALTER TABLE rag_instances 
ADD COLUMN IF NOT EXISTS is_system BOOLEAN DEFAULT false;

-- Create index for system instance queries
CREATE INDEX IF NOT EXISTS idx_rag_instances_system ON rag_instances(is_system);

-- Create trigger to update updated_at timestamp for user_group_rag_providers
CREATE OR REPLACE FUNCTION update_user_group_rag_providers_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_user_group_rag_providers_updated_at
    BEFORE UPDATE ON user_group_rag_providers
    FOR EACH ROW
    EXECUTE PROCEDURE update_user_group_rag_providers_updated_at();