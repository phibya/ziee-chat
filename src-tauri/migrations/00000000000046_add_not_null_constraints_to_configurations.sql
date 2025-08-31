-- Add NOT NULL constraints to configurations table timestamp fields
-- Update existing NULL values before adding constraints

-- Update created_at: ensure all records have default value
UPDATE configurations 
SET created_at = CURRENT_TIMESTAMP 
WHERE created_at IS NULL;

-- Update updated_at: ensure all records have default value
UPDATE configurations 
SET updated_at = CURRENT_TIMESTAMP 
WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE configurations 
ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE configurations 
ALTER COLUMN updated_at SET NOT NULL;