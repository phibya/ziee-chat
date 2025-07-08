-- Add global configuration for default language and other system-wide settings
-- This migration adds the default language configuration that will be used as fallback 
-- when users don't have their language preference set

-- Insert default language configuration into existing configurations table
INSERT INTO configurations (name, value, description) VALUES 
    ('appearance.defaultLanguage', 'en', 'Default language for the application when user language preference is not set');

-- Add comment to document this configuration
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults and system settings';