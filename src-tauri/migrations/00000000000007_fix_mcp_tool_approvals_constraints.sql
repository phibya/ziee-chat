-- Fix mcp_tool_approvals unique constraints
-- The global_unique constraint should only apply to global approvals (is_global = true)
-- Currently it applies to ALL rows, preventing conversation-specific approvals when a global one exists

-- Drop the existing constraint
ALTER TABLE mcp_tool_approvals DROP CONSTRAINT IF EXISTS mcp_tool_approvals_global_unique;

-- Recreate it as a partial unique index that only applies to global approvals
CREATE UNIQUE INDEX IF NOT EXISTS mcp_tool_approvals_global_unique
    ON mcp_tool_approvals (user_id, server_id, tool_name)
    WHERE is_global = true;
