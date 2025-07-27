-- ===============================
-- DROP PROJECT_DOCUMENTS TABLE
-- ===============================
-- This migration removes the project_documents table that was not implemented
-- and is no longer needed in the application.

-- Drop the trigger first
DROP TRIGGER IF EXISTS update_project_documents_updated_at_trigger ON project_documents;

-- Drop the indexes
DROP INDEX IF EXISTS idx_project_documents_project_id;
DROP INDEX IF EXISTS idx_project_documents_upload_status;

-- Drop the table
DROP TABLE IF EXISTS project_documents;