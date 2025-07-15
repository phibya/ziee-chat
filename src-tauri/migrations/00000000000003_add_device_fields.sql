-- Add device configuration fields to model_provider_models table
-- device_type: cpu, cuda, metal, etc.
-- device_ids: JSON array of device IDs for multi-GPU setups

ALTER TABLE model_provider_models 
    ADD COLUMN device_type VARCHAR(50) NOT NULL DEFAULT 'cpu',
    ADD COLUMN device_ids JSONB;

-- Create index on device_type for faster filtering
CREATE INDEX IF NOT EXISTS idx_model_provider_models_device_type 
    ON model_provider_models(device_type);