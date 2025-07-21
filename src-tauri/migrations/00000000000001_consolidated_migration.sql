-- ===============================
-- 1. UTILITY FUNCTIONS
-- ===============================

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- ===============================
-- 2. CORE USER SYSTEM
-- ===============================

-- Create users table (Meteor-like structure with separate tables for arrays)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(50) UNIQUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    profile JSONB,
    is_active BOOLEAN DEFAULT TRUE,
    is_protected BOOLEAN DEFAULT FALSE,
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
    is_protected BOOLEAN DEFAULT FALSE,
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

-- ===============================
-- 3. CONFIGURATION SYSTEM
-- ===============================

-- Create configuration table
CREATE TABLE configurations (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- ===============================
-- 4. MODEL PROVIDER SYSTEM
-- ===============================

-- Create model providers table
CREATE TABLE providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    provider_type VARCHAR(50) NOT NULL CHECK (provider_type IN ('local', 'openai', 'anthropic', 'groq', 'gemini', 'mistral', 'custom')),
    enabled BOOLEAN DEFAULT FALSE,
    api_key TEXT,
    base_url VARCHAR(512),
    -- Settings removed - now stored per-model in models.settings JSONB column
    built_in BOOLEAN DEFAULT FALSE,
    proxy_settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Create repositories table for model repositories (Hugging Face, etc.)
CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    url VARCHAR(512) NOT NULL,
    auth_type VARCHAR(50) NOT NULL CHECK (auth_type IN ('none', 'api_key', 'basic_auth', 'bearer_token')),
    auth_config JSONB DEFAULT '{}',
    enabled BOOLEAN DEFAULT TRUE,
    built_in BOOLEAN DEFAULT FALSE, -- true for built-in repositories like Hugging Face
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name)
);

-- Create user group model provider relationships
CREATE TABLE user_group_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(group_id, provider_id)
);

-- Create model provider models table (unified table for all model types including Candle)
CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider_id UUID NOT NULL REFERENCES providers(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    alias VARCHAR(255) NOT NULL,
    description TEXT,
    enabled BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    is_active BOOLEAN DEFAULT FALSE, -- For local start/stop state
    capabilities JSONB DEFAULT '{}',
    parameters JSONB DEFAULT '{}',
    -- Candle-specific fields
    file_size_bytes BIGINT,
    validation_status VARCHAR(50) CHECK (validation_status IN (
        'pending',        -- Initial status when model is created
        'await_upload',   -- For local folder uploads waiting for files
        'downloading',    -- For Hugging Face downloads in progress
        'processing',     -- While processing uploaded files
        'completed',      -- Successfully downloaded/processed
        'failed',         -- Download/processing failed
        'valid',          -- After successful validation
        'invalid',        -- After failed validation
        'error',          -- General error state
        'validation_warning' -- Downloaded but with validation warnings
    )),
    validation_issues JSONB,
    -- Model performance and device settings moved to settings JSONB field
    settings JSONB DEFAULT '{}',
    -- Port number where the model server is running (for local models)
    port INTEGER,
    -- Process ID of the running model server (for local models)
    pid INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT models_alias_not_empty CHECK (alias != ''),
    CONSTRAINT models_provider_id_name_unique UNIQUE (provider_id, name)
);

-- ===============================
-- 5. MODEL FILES SYSTEM
-- ===============================

-- Create model_files table for tracking individual files within models
CREATE TABLE model_files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    filename VARCHAR(500) NOT NULL,
    file_path VARCHAR(1000) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    file_type VARCHAR(50) NOT NULL,
    upload_status VARCHAR(50) DEFAULT 'pending' CHECK (upload_status IN ('pending', 'uploading', 'completed', 'failed')),
    uploaded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(model_id, filename)
);

-- ===============================
-- 6. ASSISTANTS SYSTEM
-- ===============================

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

-- ===============================
-- 7. CHAT SYSTEM WITH BRANCHING
-- ===============================

-- Create conversations table
CREATE TABLE conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    assistant_id UUID REFERENCES assistants(id),
    model_id UUID REFERENCES models(id),
    active_branch_id UUID, -- Will be set after branches table is created
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create branches table for proper branching system
CREATE TABLE branches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Add foreign key constraint for active_branch_id
ALTER TABLE conversations 
ADD CONSTRAINT fk_conversations_active_branch_id 
FOREIGN KEY (active_branch_id) REFERENCES branches(id) ON DELETE SET NULL;

-- Create messages table without branch relationship
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    role VARCHAR(20) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    originated_from_id UUID, -- Reference to the original message this was edited from
    edit_count INTEGER DEFAULT 0, -- Number of times this message lineage has been edited
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Create branch_messages table to manage branch-message relationships
CREATE TABLE branch_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_clone BOOLEAN NOT NULL DEFAULT FALSE,
    UNIQUE(branch_id, message_id)
);

-- Create message_metadata table for additional information
CREATE TABLE message_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(message_id, key)
);

-- Create conversation_metadata table
CREATE TABLE conversation_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(conversation_id, key)
);

-- ===============================
-- 8. PROJECTS SYSTEM
-- ===============================

-- Projects table
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    is_private BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Project documents table for uploaded files
CREATE TABLE project_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    file_name VARCHAR(255) NOT NULL,
    file_path VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    content_text TEXT, -- Extracted text content for search/chat
    upload_status VARCHAR(50) NOT NULL DEFAULT 'uploaded', -- uploaded, processing, processed, failed
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Project chat conversations (extends the existing conversations table)
CREATE TABLE project_conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(project_id, conversation_id)
);

-- ===============================
-- 9. INDEXES FOR PERFORMANCE
-- ===============================

-- Users and related tables indexes
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_created_at ON users(created_at);
CREATE INDEX idx_users_profile ON users USING GIN(profile);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_users_is_protected ON users(is_protected);
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
CREATE INDEX idx_user_groups_is_protected ON user_groups(is_protected);
CREATE INDEX idx_user_groups_is_active ON user_groups(is_active);
CREATE INDEX idx_user_groups_permissions ON user_groups USING GIN(permissions);

CREATE INDEX idx_user_group_memberships_user_id ON user_group_memberships(user_id);
CREATE INDEX idx_user_group_memberships_group_id ON user_group_memberships(group_id);
CREATE INDEX idx_user_group_memberships_assigned_by ON user_group_memberships(assigned_by);

CREATE INDEX idx_user_group_providers_group_id ON user_group_providers(group_id);
CREATE INDEX idx_user_group_providers_provider_id ON user_group_providers(provider_id);

-- Configuration indexes
CREATE INDEX idx_configurations_name ON configurations(name);
CREATE INDEX idx_configurations_value ON configurations USING GIN(value);

-- User settings indexes
CREATE INDEX idx_user_settings_user_id ON user_settings(user_id);
CREATE INDEX idx_user_settings_key ON user_settings(key);
CREATE INDEX idx_user_settings_user_id_key ON user_settings(user_id, key);
CREATE INDEX idx_user_settings_value ON user_settings USING GIN(value);

-- Model provider indexes
CREATE INDEX idx_providers_provider_type ON providers(provider_type);
CREATE INDEX idx_providers_enabled ON providers(enabled);
CREATE INDEX idx_models_provider_id ON models(provider_id);
CREATE INDEX idx_models_enabled ON models(enabled);
CREATE INDEX idx_models_validation_status ON models(validation_status);
CREATE INDEX idx_models_file_size_bytes ON models(file_size_bytes);

CREATE INDEX idx_model_files_model_id ON model_files(model_id);
CREATE INDEX idx_model_files_upload_status ON model_files(upload_status);

-- Assistant indexes
CREATE INDEX idx_assistants_created_by ON assistants(created_by);
CREATE INDEX idx_assistants_is_template ON assistants(is_template);
CREATE INDEX idx_assistants_is_default ON assistants(is_default);
CREATE INDEX idx_assistants_is_active ON assistants(is_active);
CREATE INDEX idx_assistants_name ON assistants(name);

-- Conversation and branching indexes
CREATE INDEX idx_conversations_user_id ON conversations(user_id);
CREATE INDEX idx_conversations_assistant_id ON conversations(assistant_id);
CREATE INDEX idx_conversations_model_id ON conversations(model_id);
CREATE INDEX idx_conversations_active_branch_id ON conversations(active_branch_id);
CREATE INDEX idx_conversations_created_at ON conversations(created_at);

CREATE INDEX idx_branches_conversation_id ON branches(conversation_id);

-- Message indexes
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_role ON messages(role);
CREATE INDEX idx_messages_created_at ON messages(created_at);
CREATE INDEX idx_messages_originated_from_id ON messages(originated_from_id);
CREATE INDEX idx_messages_updated_at ON messages(updated_at);

-- Branch-message relationship indexes
CREATE INDEX idx_branch_messages_branch_id ON branch_messages(branch_id);
CREATE INDEX idx_branch_messages_message_id ON branch_messages(message_id);
CREATE INDEX idx_branch_messages_created_at ON branch_messages(branch_id, created_at);
CREATE INDEX idx_branch_messages_is_clone ON branch_messages(is_clone);

CREATE INDEX idx_message_metadata_message_id ON message_metadata(message_id);
CREATE INDEX idx_conversation_metadata_conversation_id ON conversation_metadata(conversation_id);

-- Project indexes
CREATE INDEX idx_projects_user_id ON projects(user_id);
CREATE INDEX idx_projects_created_at ON projects(created_at DESC);
CREATE INDEX idx_projects_updated_at ON projects(updated_at DESC);
CREATE INDEX idx_project_documents_project_id ON project_documents(project_id);
CREATE INDEX idx_project_documents_upload_status ON project_documents(upload_status);
CREATE INDEX idx_project_conversations_project_id ON project_conversations(project_id);
CREATE INDEX idx_project_conversations_conversation_id ON project_conversations(conversation_id);

-- ===============================
-- 10. TRIGGERS AND FUNCTIONS
-- ===============================

-- Function to set default originated_from_id for new messages
CREATE OR REPLACE FUNCTION set_default_originated_from_id()
RETURNS TRIGGER AS $$
BEGIN
    -- If originated_from_id is not set, set it to the message's own ID
    IF NEW.originated_from_id IS NULL THEN
        NEW.originated_from_id = NEW.id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to update conversation timestamp when messages change
CREATE OR REPLACE FUNCTION update_conversation_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE conversations 
    SET updated_at = CURRENT_TIMESTAMP 
    WHERE id = NEW.conversation_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to update projects updated_at timestamp
CREATE OR REPLACE FUNCTION update_projects_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

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

CREATE TRIGGER update_configurations_updated_at
    BEFORE UPDATE ON configurations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_providers_updated_at 
    BEFORE UPDATE ON providers
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_repositories_updated_at 
    BEFORE UPDATE ON repositories
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_models_updated_at 
    BEFORE UPDATE ON models
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

-- Trigger to automatically set originated_from_id for new messages
CREATE TRIGGER trigger_set_default_originated_from_id
    BEFORE INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION set_default_originated_from_id();

-- Trigger to update conversation timestamp when messages change
CREATE TRIGGER update_conversation_on_message
    AFTER INSERT OR UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_conversation_timestamp();

-- Trigger for messages updated_at
CREATE TRIGGER update_messages_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Triggers for projects
CREATE TRIGGER update_projects_updated_at_trigger
    BEFORE UPDATE ON projects
    FOR EACH ROW
    EXECUTE FUNCTION update_projects_updated_at();

CREATE TRIGGER update_project_documents_updated_at_trigger
    BEFORE UPDATE ON project_documents
    FOR EACH ROW
    EXECUTE FUNCTION update_projects_updated_at();

-- ===============================
-- 11. DEFAULT DATA
-- ===============================

-- Insert default configuration values
INSERT INTO configurations (name, value, description) VALUES 
    ('is_initialized', 'false', 'Indicates whether the application has been initialized'),
    ('enable_user_registration', 'true', 'Controls whether new user registration is enabled'),
    ('appearance.defaultLanguage', '"en"', 'Default language for the application when user language preference is not set'),
    ('proxy', '{"enabled": false, "url": "", "username": "", "password": "", "no_proxy": "", "ignore_ssl_certificates": false, "proxy_ssl": false, "proxy_host_ssl": false, "peer_ssl": false, "host_ssl": false}', 'Global HTTP proxy configuration');

-- Create default admin group with wildcard permissions
INSERT INTO user_groups (name, description, permissions, is_protected, is_active)
VALUES (
    'admin',
    'Administrator group with full permissions',
    '["*"]',
    TRUE,
    TRUE
);

-- Create default user group with basic permissions
INSERT INTO user_groups (name, description, permissions, is_protected, is_active)
VALUES (
    'user',
    'Default user group with basic permissions',
    '["chat::use", "profile::edit", "settings::read", "settings::edit", "settings::delete"]',
    TRUE,
    TRUE
);

-- Insert default model providers
INSERT INTO providers (name, provider_type, enabled, built_in, base_url) VALUES
('Local', 'local', false, true, null),
('OpenAI', 'openai', false, true, 'https://api.openai.com/v1'),
('Anthropic', 'anthropic', false, true, 'https://api.anthropic.com/v1'),
('Groq', 'groq', false, true, 'https://api.groq.com/openai/v1'),
('Gemini', 'gemini', false, true, 'https://generativelanguage.googleapis.com/v1beta/openai'),
('Mistral', 'mistral', false, true, 'https://api.mistral.ai');

-- Insert default repositories (Hugging Face Hub and GitHub)
INSERT INTO repositories (name, url, auth_type, auth_config, enabled, built_in) VALUES
('Hugging Face Hub', 'https://huggingface.co', 'api_key', '{"api_key": "", "auth_test_api_endpoint": "https://huggingface.co/api/whoami-v2"}', true, true),
('GitHub', 'https://api.github.com', 'bearer_token', '{"token": "", "auth_test_api_endpoint": "https://api.github.com/user"}', true, true);

-- Insert default template assistant
INSERT INTO assistants (name, description, instructions, parameters, created_by, is_template, is_default, is_active) VALUES 
('Default Assistant', 'This is the default assistant.', 'You can use this assistant to chat with the LLM.', '{"stream": true, "temperature": 0.7, "frequency_penalty": 0.7, "presence_penalty": 0.7, "top_p": 0.95, "top_k": 2}', NULL, true, true, true);

-- ===============================
-- 12. TABLE COMMENTS
-- ===============================

-- Add comments to document the tables
COMMENT ON TABLE users IS 'Users table with Meteor-like structure';
COMMENT ON TABLE user_groups IS 'User groups with AWS-style permissions in array format';
COMMENT ON TABLE configurations IS 'Application configuration settings including appearance defaults, system settings, and HTTP proxy settings';
COMMENT ON TABLE user_settings IS 'User settings table for storing personal preferences like appearance, shortcuts, proxy settings, etc.';
COMMENT ON TABLE providers IS 'Model providers table for managing AI model providers like OpenAI, Anthropic, etc.';
COMMENT ON TABLE models IS 'Individual models within each provider (unified table for all model types)';
COMMENT ON TABLE model_files IS 'Individual files that make up models';
COMMENT ON TABLE assistants IS 'Assistants table with template and user-created assistants';
COMMENT ON TABLE conversations IS 'Chat conversations table';
COMMENT ON TABLE messages IS 'Chat messages table without direct branch relationship';
COMMENT ON TABLE branches IS 'Proper branching system table - each branch belongs to a conversation';
COMMENT ON TABLE branch_messages IS 'Many-to-many relationship table between branches and messages with ordering';
COMMENT ON TABLE projects IS 'Projects table for document-based chat contexts';
COMMENT ON TABLE project_documents IS 'Documents uploaded to projects';
COMMENT ON TABLE project_conversations IS 'Links conversations to projects';

-- Column comments
COMMENT ON COLUMN user_groups.permissions IS 'AWS-style permissions stored as JSON array. Supports wildcards like "users::*", "groups::*", and "*"';
COMMENT ON COLUMN user_settings.key IS 'Setting key using camelCase format (e.g., "appearance.theme", "appearance.fontSize")';
COMMENT ON COLUMN user_settings.value IS 'Setting value stored as JSONB for flexibility';
COMMENT ON COLUMN providers.provider_type IS 'Type of provider: local, openai, anthropic, groq, gemini, mistral, custom';
COMMENT ON COLUMN models.name IS 'Unique model identifier within a provider';
COMMENT ON COLUMN models.alias IS 'Human-readable display name (can be duplicated across providers)';
COMMENT ON COLUMN models.is_active IS 'Whether the model is currently running (for local models)';
COMMENT ON COLUMN models.file_size_bytes IS 'Total size of all model files in bytes - for Candle models only';
COMMENT ON COLUMN models.validation_status IS 'Status of model validation and processing - for Candle models only';
COMMENT ON COLUMN models.validation_issues IS 'JSON array of validation issues if any - for Candle models only';
COMMENT ON COLUMN providers.proxy_settings IS 'JSON object containing all proxy configuration settings including enabled, url, username, password, no_proxy, SSL settings, etc.';
COMMENT ON COLUMN model_files.filename IS 'Original filename of the uploaded file';
COMMENT ON COLUMN model_files.file_path IS 'Storage path of the file in the filesystem';
COMMENT ON COLUMN model_files.file_type IS 'Type of file (e.g., model, tokenizer, config, safetensors)';
COMMENT ON COLUMN model_files.upload_status IS 'Upload status: pending, uploading, completed, failed';
COMMENT ON TABLE models IS 'Individual models within each provider (unified table for all model types)';
COMMENT ON TABLE model_files IS 'Individual files that make up models';
COMMENT ON TABLE providers IS 'Model providers (connection/authentication only, settings moved to individual models)';
COMMENT ON COLUMN models.port IS 'Port number where the model server is running (for Candle models only)';
COMMENT ON COLUMN models.settings IS 'Model-specific performance settings (moved from providers table)';
COMMENT ON COLUMN assistants.is_template IS 'Whether this assistant is a template (admin-created) that can be cloned by users';
COMMENT ON COLUMN assistants.is_default IS 'Whether this template assistant is automatically cloned for new users';
COMMENT ON COLUMN assistants.created_by IS 'User who created this assistant (NULL for system/template assistants)';
COMMENT ON COLUMN conversations.assistant_id IS 'Assistant used in this conversation';
COMMENT ON COLUMN conversations.model_id IS 'Specific model used in this conversation';
COMMENT ON COLUMN conversations.active_branch_id IS 'Currently active branch for this conversation';
COMMENT ON COLUMN messages.originated_from_id IS 'Original message ID this was edited from (for tracking edit lineage)';
COMMENT ON COLUMN messages.edit_count IS 'Number of times this message lineage has been edited';
COMMENT ON COLUMN messages.role IS 'Message role: user, assistant, or system';
COMMENT ON COLUMN branch_messages.branch_id IS 'Reference to the branch containing this message';
COMMENT ON COLUMN branch_messages.message_id IS 'Reference to the message in this branch';
COMMENT ON COLUMN branch_messages.is_clone IS 'Indicates whether this message is a clone (belongs to multiple branches) or is unique to this branch';

-- Constraint comments
COMMENT ON CONSTRAINT models_provider_id_name_unique ON models IS 'Ensures model IDs (name) are unique per provider, while allowing duplicate display names (alias) across providers';