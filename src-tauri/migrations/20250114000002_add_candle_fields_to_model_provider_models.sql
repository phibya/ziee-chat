-- Migration to add Candle-specific fields to model_provider_models table
-- This allows both regular models and Candle models to use the same table
-- Date: 2025-01-14

-- Add the missing columns from uploaded_models to model_provider_models
ALTER TABLE model_provider_models 
ADD COLUMN IF NOT EXISTS architecture VARCHAR(100),
ADD COLUMN IF NOT EXISTS quantization VARCHAR(50),
ADD COLUMN IF NOT EXISTS file_size_bytes BIGINT DEFAULT 0,
ADD COLUMN IF NOT EXISTS checksum VARCHAR(64),
ADD COLUMN IF NOT EXISTS validation_status VARCHAR(50) DEFAULT 'pending',
ADD COLUMN IF NOT EXISTS validation_issues JSONB;

-- Add the validation_status check constraint
ALTER TABLE model_provider_models 
ADD CONSTRAINT model_provider_models_validation_status_check 
CHECK (validation_status IN (
    'pending',        -- Initial status when model is created
    'await_upload',   -- For local folder uploads waiting for files
    'downloading',    -- For Hugging Face downloads in progress
    'processing',     -- While processing uploaded files
    'completed',      -- Successfully downloaded/processed
    'failed',         -- Download/processing failed
    'valid',          -- After successful validation
    'invalid',        -- After failed validation
    'error',          -- General error state
    'validation_warning' -- Downloaded but with validation warnings
));

-- Add indexes for the new columns
CREATE INDEX IF NOT EXISTS idx_model_provider_models_validation_status ON model_provider_models(validation_status);
CREATE INDEX IF NOT EXISTS idx_model_provider_models_architecture ON model_provider_models(architecture);
CREATE INDEX IF NOT EXISTS idx_model_provider_models_file_size_bytes ON model_provider_models(file_size_bytes);

-- Update column comments
COMMENT ON COLUMN model_provider_models.architecture IS 'Model architecture type (e.g., llama, mistral, gemma) - for Candle models only';
COMMENT ON COLUMN model_provider_models.quantization IS 'Quantization format (e.g., q4_0, q8_0, fp16, fp32) - for Candle models only';
COMMENT ON COLUMN model_provider_models.file_size_bytes IS 'Total size of all model files in bytes - for Candle models only';
COMMENT ON COLUMN model_provider_models.checksum IS 'SHA-256 checksum of the model files for integrity verification - for Candle models only';
COMMENT ON COLUMN model_provider_models.validation_status IS 'Status of model validation and processing - for Candle models only';
COMMENT ON COLUMN model_provider_models.validation_issues IS 'JSON array of validation issues if any - for Candle models only';