-- Create MCP execution status enum
CREATE TYPE mcp_execution_status AS ENUM ('pending', 'running', 'completed', 'failed', 'cancelled', 'timeout');

-- Create mcp_servers table
CREATE TABLE mcp_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR NOT NULL,
    display_name VARCHAR NOT NULL,
    description TEXT,
    enabled BOOLEAN NOT NULL DEFAULT true,
    is_system BOOLEAN NOT NULL DEFAULT false,
    transport_type VARCHAR NOT NULL DEFAULT 'stdio',
    command TEXT,
    args JSONB DEFAULT '[]',
    environment_variables JSONB DEFAULT '{}',
    url TEXT,
    headers JSONB DEFAULT '{}',
    timeout_seconds INTEGER DEFAULT 30,
    status VARCHAR NOT NULL DEFAULT 'stopped',
    is_active BOOLEAN NOT NULL DEFAULT false,
    last_health_check TIMESTAMP WITH TIME ZONE,
    restart_count INTEGER NOT NULL DEFAULT 0,
    last_restart_at TIMESTAMP WITH TIME ZONE,
    max_restart_attempts INTEGER DEFAULT 3,
    process_id INTEGER,
    port INTEGER,
    tools_discovered_at TIMESTAMP WITH TIME ZONE,
    tool_count INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT mcp_servers_name_unique_per_user_or_system
        UNIQUE (name, user_id, is_system),
    CONSTRAINT mcp_servers_system_no_user
        CHECK ((is_system = true AND user_id IS NULL) OR (is_system = false)),
    CONSTRAINT mcp_servers_command_or_url
        CHECK (
            (transport_type = 'stdio' AND command IS NOT NULL) OR
            (transport_type IN ('http', 'sse') AND url IS NOT NULL)
        )
);

-- Create user_group_mcp_servers table
CREATE TABLE user_group_mcp_servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    group_id UUID NOT NULL REFERENCES user_groups(id) ON DELETE CASCADE,
    server_id UUID NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
    assigned_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    assigned_by UUID NOT NULL REFERENCES users(id),
    UNIQUE(group_id, server_id)
);

-- Create mcp_tools_cache table
CREATE TABLE mcp_tools_cache (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
    tool_name VARCHAR NOT NULL,
    tool_description TEXT,
    input_schema JSONB NOT NULL DEFAULT '{}',
    discovered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,
    usage_count INTEGER NOT NULL DEFAULT 0,
    UNIQUE(server_id, tool_name)
);

-- Create mcp_tool_approvals table
CREATE TABLE mcp_tool_approvals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE, -- NULL for global approvals
    server_id UUID NOT NULL REFERENCES mcp_servers(id) ON DELETE CASCADE,
    tool_name VARCHAR NOT NULL,
    approved BOOLEAN NOT NULL DEFAULT false,
    auto_approve BOOLEAN NOT NULL DEFAULT false,
    is_global BOOLEAN NOT NULL DEFAULT false, -- true = global approval, false = conversation-specific
    approved_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Constraints based on approval type
    CONSTRAINT mcp_tool_approvals_global_check
        CHECK ((is_global = true AND conversation_id IS NULL) OR (is_global = false AND conversation_id IS NOT NULL)),

    -- Global approvals: one per user+server+tool (for ON CONFLICT support)
    CONSTRAINT mcp_tool_approvals_global_unique
        UNIQUE (user_id, server_id, tool_name) DEFERRABLE INITIALLY DEFERRED,

    -- Conversation approvals: one per user+conversation+server+tool (for ON CONFLICT support)
    CONSTRAINT mcp_tool_approvals_conversation_unique
        UNIQUE (user_id, conversation_id, server_id, tool_name) DEFERRABLE INITIALLY DEFERRED
);

-- Create mcp_execution_logs table
CREATE TABLE mcp_execution_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    server_id UUID NOT NULL REFERENCES mcp_servers(id),
    thread_id UUID,
    tool_name VARCHAR NOT NULL,
    tool_parameters JSONB,
    execution_result JSONB,
    status mcp_execution_status NOT NULL,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    duration_ms INTEGER,
    error_message TEXT,
    error_code VARCHAR,
    request_id UUID,
    correlation_id UUID
);

-- Create indexes for performance
CREATE INDEX idx_mcp_servers_user_id ON mcp_servers(user_id);
CREATE INDEX idx_mcp_servers_is_system ON mcp_servers(is_system);
CREATE INDEX idx_mcp_servers_status ON mcp_servers(status);
CREATE INDEX idx_mcp_servers_is_active ON mcp_servers(is_active);

CREATE INDEX idx_user_group_mcp_servers_group_id ON user_group_mcp_servers(group_id);
CREATE INDEX idx_user_group_mcp_servers_server_id ON user_group_mcp_servers(server_id);

CREATE INDEX idx_mcp_tools_cache_server_id ON mcp_tools_cache(server_id);
CREATE INDEX idx_mcp_tools_cache_tool_name ON mcp_tools_cache(tool_name);

CREATE INDEX idx_mcp_tool_approvals_user_global ON mcp_tool_approvals(user_id, server_id, tool_name) WHERE is_global = true;
CREATE INDEX idx_mcp_tool_approvals_user_conversation ON mcp_tool_approvals(user_id, conversation_id) WHERE is_global = false;
CREATE INDEX idx_mcp_tool_approvals_server_tool ON mcp_tool_approvals(server_id, tool_name);
CREATE INDEX idx_mcp_tool_approvals_expires_at ON mcp_tool_approvals(expires_at) WHERE expires_at IS NOT NULL;

CREATE INDEX idx_mcp_execution_logs_user_id ON mcp_execution_logs(user_id);
CREATE INDEX idx_mcp_execution_logs_server_id ON mcp_execution_logs(server_id);
CREATE INDEX idx_mcp_execution_logs_started_at ON mcp_execution_logs(started_at);
CREATE INDEX idx_mcp_execution_logs_status ON mcp_execution_logs(status);
CREATE INDEX idx_mcp_execution_logs_thread_id ON mcp_execution_logs(thread_id);

-- Insert default system servers
INSERT INTO mcp_servers (name, display_name, description, transport_type, is_system, enabled, command, args, environment_variables) VALUES
('filesystem', 'Filesystem Access', 'Access local filesystem operations', 'stdio', true, false, 'npx', '["-y", "@modelcontextprotocol/server-filesystem"]', '{}'),
('fetch', 'Web Fetch', 'Fetch content from web URLs', 'stdio', true, false, 'uvx', '["mcp-server-fetch"]', '{}'),
('browser', 'Browser Automation', 'Automate browser interactions', 'stdio', true, false, 'npx', '["@browsermcp/mcp"]', '{}'),
('git', 'Git Operations', 'Git repository operations', 'stdio', true, false, 'npx', '["-y", "mcp-git-server"]', '{}');