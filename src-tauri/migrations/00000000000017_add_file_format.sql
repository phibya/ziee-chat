-- Add file_format column to models table
-- This column stores the model file format to help with engine selection

-- Add the file_format column
ALTER TABLE models 
ADD COLUMN file_format VARCHAR(20) NOT NULL DEFAULT 'safetensors';

-- Update existing models to have safetensors as default
UPDATE models 
SET file_format = 'safetensors' 
WHERE file_format IS NULL OR file_format = '';

-- Add check constraint to ensure valid file formats
ALTER TABLE models 
ADD CONSTRAINT check_file_format 
CHECK (file_format IN ('safetensors', 'gguf', 'bin', 'pt', 'pth', 'onnx'));

-- Add index for performance
CREATE INDEX idx_models_file_format ON models(file_format);

-- Add comment for documentation
COMMENT ON COLUMN models.file_format IS 'Model file format: safetensors, gguf, bin, pt, pth, onnx';