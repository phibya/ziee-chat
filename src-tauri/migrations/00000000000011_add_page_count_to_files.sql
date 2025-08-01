-- Add page_count column to files table
-- This column stores the number of high-quality images generated for the file

ALTER TABLE files ADD COLUMN page_count INTEGER DEFAULT 0;

-- Add index for page_count for efficient querying
CREATE INDEX idx_files_page_count ON files(page_count);

-- Update existing files to have default page_count of 0
UPDATE files SET page_count = 0 WHERE page_count IS NULL;

-- Add comment for the new column
COMMENT ON COLUMN files.page_count IS 'Number of high-quality images/pages generated for this file';