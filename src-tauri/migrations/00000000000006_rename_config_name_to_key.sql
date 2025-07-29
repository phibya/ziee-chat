-- Migration to rename 'name' column to 'key' in configurations table
-- This provides more semantic clarity for configuration identifiers

-- Rename the column from 'name' to 'key'
ALTER TABLE configurations RENAME COLUMN name TO key;

-- Update the unique constraint to reflect the new column name
-- Note: PostgreSQL automatically updates constraints when columns are renamed,
-- but we explicitly recreate it for clarity and to ensure proper naming
ALTER TABLE configurations DROP CONSTRAINT configurations_name_key;
ALTER TABLE configurations ADD CONSTRAINT configurations_key_key UNIQUE (key);