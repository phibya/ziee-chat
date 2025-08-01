-- Remove document extraction configuration data
-- This migration removes all document_extraction.* configuration entries from the configurations table

-- Delete all document extraction configuration entries
DELETE FROM configurations WHERE key LIKE 'document_extraction.%';

-- Add comment documenting the removal
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults, system settings, and HTTP proxy settings. Document extraction settings have been removed as of migration 12.';