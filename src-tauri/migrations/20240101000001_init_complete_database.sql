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
    proxy_enabled BOOLEAN DEFAULT FALSE,
    proxy_url VARCHAR(512) DEFAULT '',
    proxy_username VARCHAR(255) DEFAULT '',
    proxy_password TEXT DEFAULT '',
    proxy_no_proxy TEXT DEFAULT '',
    proxy_ignore_ssl_certificates BOOLEAN DEFAULT FALSE,
    proxy_ssl BOOLEAN DEFAULT FALSE,
    proxy_host_ssl BOOLEAN DEFAULT FALSE,
    proxy_peer_ssl BOOLEAN DEFAULT FALSE,
    proxy_host_ssl_verify BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create user group model provider relationships
CREATE TABLE user_group_model_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES model_providers(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(group_id, provider_id)
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

-- Create assistants table
CREATE TABLE assistants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    instructions TEXT,
    parameters JSONB DEFAULT '{}',
    created_by UUID REFERENCES users(id),
    is_template BOOLEAN DEFAULT false,
    is_default BOOLEAN DEFAULT false,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assistant_id UUID REFERENCES assistants(id),
    model_provider_id UUID REFERENCES model_providers(id),
    model_id UUID REFERENCES model_provider_models(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create messages table
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_message_id UUID REFERENCES messages(id),
    content TEXT NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    branch_index INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
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

CREATE INDEX idx_user_group_model_providers_group_id ON user_group_model_providers(group_id);
CREATE INDEX idx_user_group_model_providers_provider_id ON user_group_model_providers(provider_id);

CREATE INDEX idx_configurations_name ON configurations(name);

CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
CREATE INDEX idx_user_settings_key ON user_settings(key);
CREATE INDEX idx_user_settings_user_id_key ON user_settings(user_id, key);
CREATE INDEX idx_user_settings_value ON user_settings USING GIN(value);

CREATE INDEX idx_model_providers_provider_type ON model_providers(provider_type);
CREATE INDEX idx_model_providers_enabled ON model_providers(enabled);
CREATE INDEX idx_model_providers_proxy_enabled ON model_providers(proxy_enabled);
CREATE INDEX idx_model_provider_models_provider_id ON model_provider_models(provider_id);
CREATE INDEX idx_model_provider_models_enabled ON model_provider_models(enabled);

CREATE INDEX idx_assistants_created_by ON assistants(created_by);
CREATE INDEX idx_assistants_is_template ON assistants(is_template);
CREATE INDEX idx_assistants_is_default ON assistants(is_default);
CREATE INDEX idx_assistants_is_active ON assistants(is_active);
CREATE INDEX idx_assistants_name ON assistants(name);

CREATE INDEX idx_conversations_user_id ON conversations(user_id);
CREATE INDEX idx_conversations_assistant_id ON conversations(assistant_id);
CREATE INDEX idx_conversations_model_provider_id ON conversations(model_provider_id);
CREATE INDEX idx_conversations_model_id ON conversations(model_id);
CREATE INDEX idx_conversations_created_at ON conversations(created_at);

CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_parent_message_id ON messages(parent_message_id);
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_branch_index ON messages(branch_index);

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

CREATE TRIGGER update_assistants_updated_at
    BEFORE UPDATE ON assistants
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default configuration values
INSERT INTO configurations (name, value, description) VALUES 
    ('is_initialized', 'false', 'Indicates whether the application has been initialized'),
    ('enable_user_registration', 'true', 'Controls whether new user registration is enabled'),
    ('appearance.defaultLanguage', 'en', 'Default language for the application when user language preference is not set'),
    ('proxy.enabled', 'false', 'Enable global HTTP proxy for the application'),
    ('proxy.url', '', 'Global HTTP proxy URL'),
    ('proxy.username', '', 'Global HTTP proxy username'),
    ('proxy.password', '', 'Global HTTP proxy password'),
    ('proxy.noProxy', '', 'Global HTTP proxy no-proxy list (comma-separated)'),
    ('proxy.ignoreSslCertificates', 'false', 'Ignore SSL certificates for proxy'),
    ('proxy.proxySsl', 'false', 'Validate SSL certificate when connecting to proxy'),
    ('proxy.proxyHostSsl', 'false', 'Validate SSL certificate of proxy host'),
    ('proxy.peerSsl', 'false', 'Validate SSL certificates of peer connections'),
    ('proxy.hostSsl', 'false', 'Validate SSL certificates of destination hosts');

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
INSERT INTO model_providers (name, provider_type, enabled, is_default, base_url, settings) VALUES
('Llama.cpp', 'llama.cpp', false, true, null, '{"autoUnloadOldModels": true, "contextShift": false, "continuousBatching": false, "parallelOperations": 1, "cpuThreads": -1, "threadsBatch": -1, "flashAttention": true, "caching": true, "kvCacheType": "q8_0", "mmap": true, "huggingFaceAccessToken": ""}'),
('OpenAI', 'openai', false, true, 'https://api.openai.com/v1', '{}'),
('Anthropic', 'anthropic', false, true, 'https://api.anthropic.com/v1', '{}'),
('Groq', 'groq', false, true, 'https://api.groq.com/openai/v1', '{}'),
('Gemini', 'gemini', false, true, 'https://generativelanguage.googleapis.com/v1beta/openai', '{}'),
('Mistral', 'mistral', false, true, 'https://api.mistral.ai', '{}');

-- Insert default template assistant
INSERT INTO assistants (name, description, instructions, parameters, created_by, is_template, is_default, is_active) VALUES 
('Default Assistant', 'This is the default assistant.', 'You can use this assistant to chat with the LLM.', '{"stream": true, "temperature": 0.7, "frequency_penalty": 0.7, "presence_penalty": 0.7, "top_p": 0.95, "top_k": 2}', NULL, true, true, true);

-- Add comments to document the tables
COMMENT ON TABLE users IS 'Users table with Meteor-like structure';
COMMENT ON TABLE user_groups IS 'User groups with AWS-style permissions in array format';
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults, system settings, and HTTP proxy settings';
COMMENT ON TABLE user_settings IS 'User settings table for storing personal preferences like appearance, shortcuts, proxy settings, etc.';
COMMENT ON TABLE model_providers IS 'Model providers table for managing AI model providers like OpenAI, Anthropic, etc.';
COMMENT ON TABLE model_provider_models IS 'Individual models within each provider';
COMMENT ON TABLE assistants IS 'Assistants table with template and user-created assistants';
COMMENT ON TABLE conversations IS 'Chat conversations table';
COMMENT ON TABLE messages IS 'Chat messages table with branching support';

COMMENT ON COLUMN user_groups.permissions IS 'AWS-style permissions stored as JSON array. Supports wildcards like "users::*", "groups::*", and "*"';
COMMENT ON COLUMN user_settings.key IS 'Setting key using camelCase format (e.g., "appearance.theme", "appearance.fontSize")';
COMMENT ON COLUMN user_settings.value IS 'Setting value stored as JSONB for flexibility';
COMMENT ON COLUMN model_providers.provider_type IS 'Type of provider: llama.cpp, openai, anthropic, groq, gemini, mistral, custom';
COMMENT ON COLUMN model_provider_models.path IS 'File path for llama.cpp models';
COMMENT ON COLUMN model_provider_models.is_active IS 'Whether the model is currently running (for llama.cpp models)';
COMMENT ON COLUMN model_providers.proxy_enabled IS 'Whether proxy is enabled for this provider';
COMMENT ON COLUMN model_providers.proxy_url IS 'Proxy URL for this provider';
COMMENT ON COLUMN model_providers.proxy_username IS 'Proxy username for authentication';
COMMENT ON COLUMN model_providers.proxy_password IS 'Proxy password for authentication';
COMMENT ON COLUMN model_providers.proxy_no_proxy IS 'Comma-separated list of hosts to bypass proxy';
COMMENT ON COLUMN model_providers.proxy_ignore_ssl_certificates IS 'Whether to ignore SSL certificate errors';
COMMENT ON COLUMN model_providers.proxy_ssl IS 'Whether to use SSL for proxy connection';
COMMENT ON COLUMN model_providers.proxy_host_ssl IS 'Whether to use SSL for host connection';
COMMENT ON COLUMN model_providers.proxy_peer_ssl IS 'Whether to use SSL for peer connection';
COMMENT ON COLUMN model_providers.proxy_host_ssl_verify IS 'Whether to verify SSL certificates for host';
COMMENT ON COLUMN assistants.is_template IS 'Whether this assistant is a template (admin-created) that can be cloned by users';
COMMENT ON COLUMN assistants.is_default IS 'Whether this template assistant is automatically cloned for new users';
COMMENT ON COLUMN assistants.created_by IS 'User who created this assistant (NULL for system/template assistants)';
COMMENT ON COLUMN conversations.assistant_id IS 'Assistant used in this conversation';
COMMENT ON COLUMN conversations.model_provider_id IS 'Model provider used in this conversation';
COMMENT ON COLUMN conversations.model_id IS 'Specific model used in this conversation';
COMMENT ON COLUMN messages.parent_message_id IS 'Parent message for branching support';
COMMENT ON COLUMN messages.branch_index IS 'Branch index for message branching';
COMMENT ON COLUMN messages.role IS 'Message role: user, assistant, or system';