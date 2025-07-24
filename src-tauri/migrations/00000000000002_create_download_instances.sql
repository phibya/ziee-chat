-- Create download_instances table for tracking model downloads
CREATE TABLE download_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    request_data JSONB NOT NULL, -- Stores all download parameters
    status VARCHAR(50) NOT NULL CHECK (status IN ('pending', 'downloading', 'completed', 'failed', 'cancelled')),
    progress_data JSONB DEFAULT '{}', -- Stores phase, current, total, message
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP WITH TIME ZONE,
    model_id UUID REFERENCES models(id) ON DELETE SET NULL, -- Nullable, filled when download completes
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better query performance
CREATE INDEX idx_download_instances_user_id ON download_instances(user_id);
CREATE INDEX idx_download_instances_provider_id ON download_instances(provider_id);
CREATE INDEX idx_download_instances_repository_id ON download_instances(repository_id);
CREATE INDEX idx_download_instances_status ON download_instances(status);
CREATE INDEX idx_download_instances_started_at ON download_instances(started_at DESC);
CREATE INDEX idx_download_instances_model_id ON download_instances(model_id);

-- Create trigger to update updated_at timestamp
CREATE TRIGGER download_instances_updated_at BEFORE UPDATE ON download_instances
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE download_instances IS 'Tracks model download requests and their progress';
COMMENT ON COLUMN download_instances.request_data IS 'JSON containing all download parameters like model_id, quantization, etc.';
COMMENT ON COLUMN download_instances.progress_data IS 'JSON with structure: {phase: string, current: number, total: number, message: string}';
COMMENT ON COLUMN download_instances.model_id IS 'References the created model entry after successful download';

-- Down migration
-- DROP TRIGGER IF EXISTS download_instances_updated_at ON download_instances;
-- DROP TABLE IF EXISTS download_instances;