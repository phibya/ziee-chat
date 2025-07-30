-- Migration to remove 'file_path' column from files table
-- File paths are now computed dynamically using FILE_STORAGE.get_original_path()
-- This improves portability and removes dependency on absolute paths

-- Drop the file_path column from the files table
ALTER TABLE files DROP COLUMN file_path;