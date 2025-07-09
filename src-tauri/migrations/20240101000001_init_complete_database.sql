-- Complete database initialization with all tables and features
-- This migration creates all required tables and sets up the complete system

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create users table (Meteor-like structure with separate tables for arrays)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    profile JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    last_login_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create user_emails table (for the emails array)
CREATE TABLE user_emails (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    address VARCHAR(255) NOT NULL UNIQUE,
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create user_services table (for the services object)
CREATE TABLE user_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    service_name VARCHAR(50) NOT NULL,
    service_data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, service_name)
);

-- Create user_login_tokens table (for resume.loginTokens array)
CREATE TABLE user_login_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    when_created BIGINT NOT NULL, -- Unix timestamp in milliseconds
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create user groups table with AWS-style permissions
CREATE TABLE user_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    permissions JSONB DEFAULT '[]', -- Array format for AWS-style permissions
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Create user_group_memberships table (many-to-many relationship)
CREATE TABLE user_group_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    assigned_by UUID REFERENCES users(id),
    UNIQUE(user_id, group_id)
);

-- Create configuration table
CREATE TABLE configurations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    value TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create user settings table to store personal user preferences
CREATE TABLE user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(user_id, key)
);

-- Create model providers table
CREATE TABLE model_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL CHECK (provider_type IN ('llama.cpp', 'openai', 'anthropic', 'groq', 'gemini', 'mistral', 'custom')),
    enabled BOOLEAN DEFAULT FALSE,
    api_key TEXT,
    base_url VARCHAR(512),
    settings JSONB DEFAULT '{}',
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create model provider models table
CREATE TABLE model_provider_models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES model_providers(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    path VARCHAR(1024), -- For llama.cpp models
    enabled BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT FALSE, -- For llama.cpp start/stop state
    capabilities JSONB DEFAULT '{}',
    parameters JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create indexes for better performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_profile ON users USING GIN(profile);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_users_last_login_at ON users(last_login_at);
CREATE INDEX idx_users_updated_at ON users(updated_at);

CREATE INDEX idx_user_emails_user_id ON user_emails(user_id);
CREATE INDEX idx_user_emails_address ON user_emails(address);
CREATE INDEX idx_user_emails_verified ON user_emails(verified);

CREATE INDEX idx_user_services_user_id ON user_services(user_id);
CREATE INDEX idx_user_services_service_name ON user_services(service_name);
CREATE INDEX idx_user_services_data ON user_services USING GIN(service_data);

CREATE INDEX idx_user_login_tokens_user_id ON user_login_tokens(user_id);
CREATE INDEX idx_user_login_tokens_token ON user_login_tokens(token);
CREATE INDEX idx_user_login_tokens_expires_at ON user_login_tokens(expires_at);

CREATE INDEX idx_user_groups_name ON user_groups(name);
CREATE INDEX idx_user_groups_is_active ON user_groups(is_active);
CREATE INDEX idx_user_groups_permissions ON user_groups USING GIN(permissions);

CREATE INDEX idx_user_group_memberships_user_id ON user_group_memberships(user_id);
CREATE INDEX idx_user_group_memberships_group_id ON user_group_memberships(group_id);
CREATE INDEX idx_user_group_memberships_assigned_by ON user_group_memberships(assigned_by);

CREATE INDEX idx_configurations_name ON configurations(name);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
CREATE INDEX idx_user_settings_key ON user_settings(key);
CREATE INDEX idx_user_settings_user_id_key ON user_settings(user_id, key);
CREATE INDEX idx_user_settings_value ON user_settings USING GIN(value);

CREATE INDEX idx_model_providers_provider_type ON model_providers(provider_type);
CREATE INDEX idx_model_providers_enabled ON model_providers(enabled);
CREATE INDEX idx_model_provider_models_provider_id ON model_provider_models(provider_id);
CREATE INDEX idx_model_provider_models_enabled ON model_provider_models(enabled);

-- Create triggers for updated_at columns
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_groups_updated_at
    BEFORE UPDATE ON user_groups
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_settings_updated_at
    BEFORE UPDATE ON user_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_model_providers_updated_at 
    BEFORE UPDATE ON model_providers
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_model_provider_models_updated_at 
    BEFORE UPDATE ON model_provider_models
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default configuration values
INSERT INTO configurations (name, value, description) VALUES 
    ('is_initialized', 'false', 'Indicates whether the application has been initialized'),
    ('enable_user_registration', 'true', 'Controls whether new user registration is enabled'),
    ('appearance.defaultLanguage', 'en', 'Default language for the application when user language preference is not set');

-- Create default admin group with wildcard permissions
INSERT INTO user_groups (name, description, permissions, is_active)
VALUES (
    'admin',
    'Administrator group with full permissions',
    '["*"]',
    TRUE
);

-- Create default user group with basic permissions
INSERT INTO user_groups (name, description, permissions, is_active)
VALUES (
    'user',
    'Default user group with basic permissions',
    '["chat::use", "profile::edit", "settings::read", "settings::edit", "settings::delete"]',
    TRUE
);

-- Insert default model providers
INSERT INTO model_providers (id, name, provider_type, enabled, is_default, base_url, settings) VALUES
('11111111-1111-1111-1111-111111111111', 'Llama.cpp', 'llama.cpp', false, true, null, '{"autoUnloadOldModels": true, "contextShift": false, "continuousBatching": false, "parallelOperations": 1, "cpuThreads": -1, "threadsBatch": -1, "flashAttention": true, "caching": true, "kvCacheType": "q8_0", "mmap": true, "huggingFaceAccessToken": ""}'),
('22222222-2222-2222-2222-222222222222', 'OpenAI', 'openai', false, true, 'https://api.openai.com/v1', '{}'),
('33333333-3333-3333-3333-333333333333', 'Anthropic', 'anthropic', false, true, 'https://api.anthropic.com/v1', '{}'),
('44444444-4444-4444-4444-444444444444', 'Groq', 'groq', false, true, 'https://api.groq.com/openai/v1', '{}'),
('55555555-5555-5555-5555-555555555555', 'Gemini', 'gemini', false, true, 'https://generativelanguage.googleapis.com/v1beta/openai', '{}'),
('66666666-6666-6666-6666-666666666666', 'Mistral', 'mistral', false, true, 'https://api.mistral.ai', '{}');

-- Add comments to document the tables
COMMENT ON TABLE users IS 'Users table with Meteor-like structure';
COMMENT ON TABLE user_groups IS 'User groups with AWS-style permissions in array format';
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults and system settings';
COMMENT ON TABLE user_settings IS 'User settings table for storing personal preferences like appearance, shortcuts, proxy settings, etc.';
COMMENT ON TABLE model_providers IS 'Model providers table for managing AI model providers like OpenAI, Anthropic, etc.';
COMMENT ON TABLE model_provider_models IS 'Individual models within each provider';

COMMENT ON COLUMN user_groups.permissions IS 'AWS-style permissions stored as JSON array. Supports wildcards like "users::*", "groups::*", and "*"';
COMMENT ON COLUMN user_settings.key IS 'Setting key using camelCase format (e.g., "appearance.theme", "appearance.fontSize")';
COMMENT ON COLUMN user_settings.value IS 'Setting value stored as JSONB for flexibility';
COMMENT ON COLUMN model_providers.provider_type IS 'Type of provider: llama.cpp, openai, anthropic, groq, gemini, mistral, custom';
COMMENT ON COLUMN model_provider_models.path IS 'File path for llama.cpp models';
COMMENT ON COLUMN model_provider_models.is_active IS 'Whether the model is currently running (for llama.cpp models)';