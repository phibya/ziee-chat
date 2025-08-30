-- Add NOT NULL constraints to user tables to match Rust model expectations
-- This migration ensures schema consistency for SQLx macro usage

-- First, set default values for any NULL fields to prevent constraint violations

-- Update users table - set defaults for NULL values before adding constraints
UPDATE users 
SET 
    username = COALESCE(username, 'user_' || id::text) 
WHERE username IS NULL;

UPDATE users 
SET 
    is_active = COALESCE(is_active, TRUE) 
WHERE is_active IS NULL;

UPDATE users 
SET 
    is_protected = COALESCE(is_protected, FALSE) 
WHERE is_protected IS NULL;

UPDATE users 
SET 
    updated_at = COALESCE(updated_at, NOW()) 
WHERE updated_at IS NULL;

-- Update user_emails table
UPDATE user_emails 
SET 
    verified = COALESCE(verified, FALSE) 
WHERE verified IS NULL;

UPDATE user_emails 
SET 
    created_at = COALESCE(created_at, NOW()) 
WHERE created_at IS NULL;

-- Update user_services table
UPDATE user_services 
SET 
    created_at = COALESCE(created_at, NOW()) 
WHERE created_at IS NULL;

-- Update user_login_tokens table
UPDATE user_login_tokens 
SET 
    created_at = COALESCE(created_at, NOW()) 
WHERE created_at IS NULL;

-- Now add the NOT NULL constraints
ALTER TABLE users 
    ALTER COLUMN username SET NOT NULL,
    ALTER COLUMN is_active SET NOT NULL,
    ALTER COLUMN is_protected SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE user_emails
    ALTER COLUMN verified SET NOT NULL,
    ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE user_services
    ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE user_login_tokens
    ALTER COLUMN created_at SET NOT NULL;