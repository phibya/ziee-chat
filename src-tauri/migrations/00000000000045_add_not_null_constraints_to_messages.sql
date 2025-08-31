-- Add NOT NULL constraints to messages table fields
-- Update existing NULL values before adding constraints

-- Update originated_from_id: set to message id for messages that don't have an origin
UPDATE messages 
SET originated_from_id = id 
WHERE originated_from_id IS NULL;

-- Update edit_count: ensure all records have default value
UPDATE messages 
SET edit_count = 0 
WHERE edit_count IS NULL;

-- Update created_at: ensure all records have default value
UPDATE messages 
SET created_at = NOW() 
WHERE created_at IS NULL;

-- Update updated_at: ensure all records have default value
UPDATE messages 
SET updated_at = NOW() 
WHERE updated_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE messages 
ALTER COLUMN originated_from_id SET NOT NULL;

ALTER TABLE messages 
ALTER COLUMN edit_count SET NOT NULL;

ALTER TABLE messages 
ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE messages 
ALTER COLUMN updated_at SET NOT NULL;