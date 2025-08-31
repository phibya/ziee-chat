-- Add NOT NULL constraints to model_files table fields

-- Update any NULL values to appropriate defaults before adding constraints  
UPDATE model_files SET filename = 'unknown' WHERE filename IS NULL;
UPDATE model_files SET file_type = 'unknown' WHERE file_type IS NULL;
UPDATE model_files SET upload_status = 'completed' WHERE upload_status IS NULL;
UPDATE model_files SET uploaded_at = CURRENT_TIMESTAMP WHERE uploaded_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE model_files ALTER COLUMN filename SET NOT NULL;
ALTER TABLE model_files ALTER COLUMN file_type SET NOT NULL;
ALTER TABLE model_files ALTER COLUMN upload_status SET NOT NULL;
ALTER TABLE model_files ALTER COLUMN uploaded_at SET NOT NULL;