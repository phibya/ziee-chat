-- Add rag_instance_id column to files table for direct RAG association
-- This enables CASCADE deletion when RAG instances are deleted

-- Add the new column
ALTER TABLE files ADD COLUMN rag_instance_id UUID;

-- Add foreign key constraint with CASCADE deletion
ALTER TABLE files ADD CONSTRAINT fk_files_rag_instance_id 
  FOREIGN KEY (rag_instance_id) REFERENCES rag_instances(id) ON DELETE CASCADE;

-- Create index for performance
CREATE INDEX idx_files_rag_instance_id ON files(rag_instance_id);