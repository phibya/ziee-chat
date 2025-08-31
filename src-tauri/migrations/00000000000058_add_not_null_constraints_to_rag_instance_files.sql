-- Add NOT NULL constraints to rag_instance_files table for processing_status

-- Update any NULL values to appropriate defaults before adding constraints
UPDATE rag_instance_files SET processing_status = 'pending' WHERE processing_status IS NULL;

-- Add NOT NULL constraints
ALTER TABLE rag_instance_files ALTER COLUMN processing_status SET NOT NULL;