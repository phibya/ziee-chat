-- Add NOT NULL constraints to API proxy tables to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update api_proxy_server_models table
UPDATE api_proxy_server_models 
SET 
    created_at = COALESCE(created_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE created_at IS NULL OR updated_at IS NULL;

-- Update api_proxy_server_trusted_hosts table
UPDATE api_proxy_server_trusted_hosts 
SET 
    created_at = COALESCE(created_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE created_at IS NULL OR updated_at IS NULL;

-- Now add the NOT NULL constraints
ALTER TABLE api_proxy_server_models
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE api_proxy_server_trusted_hosts
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;