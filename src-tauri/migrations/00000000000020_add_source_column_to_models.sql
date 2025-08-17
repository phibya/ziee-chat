-- Add source column to models table
-- This column stores the source information for tracking model origins

-- Add the source column as JSONB to store SourceInfo
ALTER TABLE models 
ADD COLUMN source JSONB DEFAULT NULL;

-- Add check constraint to ensure valid source structure when not null
ALTER TABLE models
ADD CONSTRAINT check_source_structure 
CHECK (
    source IS NULL OR (
        source ? 'type' AND 
        (source->>'type' = 'manual' OR source->>'type' = 'hub') AND
        source ? 'id'
    )
);

-- Add index for performance on source queries
CREATE INDEX idx_models_source_type ON models((source->>'type'));
CREATE INDEX idx_models_source_hub_id ON models((source->>'id')) WHERE source->>'type' = 'hub';

-- Add comment for documentation
COMMENT ON COLUMN models.source IS 'Source information: {type: "manual"|"hub", id: string|null}';