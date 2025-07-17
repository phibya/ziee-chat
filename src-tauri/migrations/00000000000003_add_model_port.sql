-- Add port field to models table for tracking running model ports
-- This allows us to store the port number when a model is started so we can call its API

-- Add port column to models table
ALTER TABLE models ADD COLUMN port INTEGER;

-- Add comment to document the port field
COMMENT ON COLUMN models.port IS 'Port number where the model server is running (for Candle models only)';