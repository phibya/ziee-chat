-- Remove user_id from download_instances as downloads belong to the system
-- Users need edit permission on providers to start downloads

-- Drop the index on user_id first
DROP INDEX IF EXISTS idx_download_instances_user_id;

-- Drop the user_id column
ALTER TABLE download_instances DROP COLUMN IF EXISTS user_id;

-- Down migration (to revert changes)
-- ALTER TABLE download_instances ADD COLUMN user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE;
-- CREATE INDEX idx_download_instances_user_id ON download_instances(user_id);