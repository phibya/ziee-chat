-- Migration: Add Enterprise Authentication Support
-- Tables: auth_providers, user_auth_links, auth_sessions

-- Create auth_providers table
CREATE TABLE auth_providers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    provider_type VARCHAR(50) NOT NULL CHECK (provider_type IN ('local', 'ldap', 'oauth2', 'oidc', 'saml')),
    enabled BOOLEAN DEFAULT TRUE NOT NULL,
    priority INTEGER DEFAULT 0 NOT NULL,
    config JSONB DEFAULT '{}' NOT NULL,
    mapping_rules JSONB DEFAULT '{}' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Indexes for auth_providers
CREATE INDEX idx_auth_providers_type ON auth_providers(provider_type);
CREATE INDEX idx_auth_providers_enabled ON auth_providers(enabled);
CREATE INDEX idx_auth_providers_priority ON auth_providers(priority DESC);

-- Ensure only one local provider can exist
CREATE UNIQUE INDEX idx_auth_providers_unique_local
ON auth_providers(provider_type)
WHERE provider_type = 'local';

-- Create user_auth_links table
CREATE TABLE user_auth_links (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider_id UUID NOT NULL REFERENCES auth_providers(id) ON DELETE CASCADE,
    external_id VARCHAR(512) NOT NULL,
    external_username VARCHAR(255),
    external_email VARCHAR(255),
    external_metadata JSONB DEFAULT '{}',
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL,
    UNIQUE(provider_id, external_id)
);

-- Indexes for user_auth_links
CREATE INDEX idx_user_auth_links_user ON user_auth_links(user_id);
CREATE INDEX idx_user_auth_links_provider ON user_auth_links(provider_id);
CREATE INDEX idx_user_auth_links_external_id ON user_auth_links(external_id);

-- Create auth_sessions table (for OAuth/SAML flows)
CREATE TABLE auth_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_key VARCHAR(255) NOT NULL UNIQUE,
    provider_id UUID NOT NULL REFERENCES auth_providers(id) ON DELETE CASCADE,
    state VARCHAR(512) NOT NULL,
    nonce VARCHAR(512),
    code_verifier VARCHAR(512),
    redirect_uri VARCHAR(512),
    metadata JSONB DEFAULT '{}',
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

-- Indexes for auth_sessions
CREATE INDEX idx_auth_sessions_session_key ON auth_sessions(session_key);
CREATE INDEX idx_auth_sessions_provider ON auth_sessions(provider_id);
CREATE INDEX idx_auth_sessions_expires ON auth_sessions(expires_at);

-- Ensure default "user" group exists
INSERT INTO user_groups (name, description, permissions, is_protected, is_active)
VALUES (
    'user',
    'Default user group for authenticated users',
    '["chat::read", "chat::create", "files::read", "files::upload", "assistants::read"]',
    true,
    true
)
ON CONFLICT (name) DO NOTHING;

-- Insert default local auth provider
INSERT INTO auth_providers (name, provider_type, enabled, priority, config, mapping_rules)
VALUES (
    'Local Authentication',
    'local',
    TRUE,
    100,
    '{}',
    '{}'
);

-- Migrate existing users to use local auth provider
INSERT INTO user_auth_links (user_id, provider_id, external_id, external_username, external_email)
SELECT
    u.id,
    (SELECT id FROM auth_providers WHERE provider_type = 'local' LIMIT 1),
    u.id::text,
    u.username,
    e.address
FROM users u
LEFT JOIN user_emails e ON e.user_id = u.id AND e.id = (
    SELECT id FROM user_emails WHERE user_id = u.id ORDER BY created_at LIMIT 1
)
WHERE EXISTS (SELECT 1 FROM user_services s WHERE s.user_id = u.id AND s.service_name = 'password');

-- Add configuration settings for enterprise auth
INSERT INTO configurations (key, value, description) VALUES
('auth_allow_local_fallback', 'true', 'Allow local auth if external fails'),
('auth_auto_provision_users', 'true', 'Automatically create users on first login'),
('auth_sync_groups_on_login', 'true', 'Sync group memberships on each login'),
('auth_session_timeout', '15', 'Auth session timeout in minutes'),
('auth_enable_home_realm_discovery', 'true', 'Enable automatic identity provider discovery based on username'),
('auth_discovery_min_username_length', '3', 'Minimum username length before triggering provider discovery'),
('auth_default_user_group', '"user"', 'Default group for users with no group mapping'),
('auth_require_group_membership', 'false', 'Reject authentication if user has no group membership')
ON CONFLICT (key) DO NOTHING;
