-- Remove rag_instance_id column from files table to match Rust model
-- This migration ensures schema consistency for SQLx macro usage

-- Drop the rag_instance_id column from files table
ALTER TABLE files DROP COLUMN IF EXISTS rag_instance_id;