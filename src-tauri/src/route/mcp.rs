use crate::api::mcp::{approvals, servers, tools, execution};
use crate::database::models::mcp_server::MCPServer;
use crate::database::models::mcp_tool::{MCPTool, MCPToolWithServer, ToolExecutionResponse};
use servers::ServerActionResponse;
use aide::axum::{
    routing::{delete_with, get_with, post_with, put_with},
    ApiRouter,
};
use axum::{middleware, Json};

pub fn mcp_routes() -> ApiRouter {
    ApiRouter::new()
        .nest("/mcp", all_mcp_routes())
}

/// All MCP routes (user + admin operations)
fn all_mcp_routes() -> ApiRouter {
    ApiRouter::new()
        // User Server management
        .api_route(
            "/servers",
            get_with(servers::list_user_servers, |op| {
                op.description("List user's MCP servers and accessible system servers")
                    .id("Mcp.listServers")
                    .tag("mcp")
                    .response::<200, Json<servers::ListServersResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_read_middleware,
            )),
        )
        .api_route(
            "/servers",
            post_with(servers::create_user_server, |op| {
                op.description("Create new user MCP server")
                    .id("Mcp.createServer")
                    .tag("mcp")
                    .response::<201, Json<MCPServer>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_create_middleware,
            )),
        )
        .api_route(
            "/servers/{id}",
            get_with(servers::get_server, |op| {
                op.description("Get MCP server by ID")
                    .id("Mcp.getServer")
                    .tag("mcp")
                    .response::<200, Json<MCPServer>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_read_middleware,
            )),
        )
        .api_route(
            "/servers/{id}",
            put_with(servers::update_server, |op| {
                op.description("Update MCP server")
                    .id("Mcp.updateServer")
                    .tag("mcp")
                    .response::<200, Json<MCPServer>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_edit_middleware,
            )),
        )
        .api_route(
            "/servers/{id}",
            delete_with(servers::delete_server, |op| {
                op.description("Delete MCP server")
                    .id("Mcp.deleteServer")
                    .tag("mcp")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_delete_middleware,
            )),
        )
        // Server management operations
        .api_route(
            "/servers/{id}/start",
            post_with(servers::start_server, |op| {
                op.description("Start MCP server")
                    .id("Mcp.startServer")
                    .tag("mcp")
                    .response::<200, Json<ServerActionResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_edit_middleware,
            )),
        )
        .api_route(
            "/servers/{id}/stop",
            post_with(servers::stop_server, |op| {
                op.description("Stop MCP server")
                    .id("Mcp.stopServer")
                    .tag("mcp")
                    .response::<200, Json<ServerActionResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_edit_middleware,
            )),
        )
        // Tool management
        .api_route(
            "/tools",
            get_with(tools::list_user_tools, |op| {
                op.description("List available tools from user's servers")
                    .id("Mcp.listTools")
                    .tag("mcp")
                    .response::<200, Json<tools::ListToolsResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        .api_route(
            "/servers/{id}/tools",
            get_with(tools::get_server_tools, |op| {
                op.description("Get tools for specific server")
                    .id("Mcp.getServerTools")
                    .tag("mcp")
                    .response::<200, Json<Vec<MCPTool>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        // Tool execution
        .api_route(
            "/tools/find",
            get_with(tools::find_tool_by_name, |op| {
                op.description("Find tool by name across servers")
                    .id("Mcp.findTool")
                    .tag("mcp")
                    .response::<200, Json<Option<MCPToolWithServer>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        // Global tool approval management
        .api_route(
            "/servers/{server_id}/tools/{tool_name}/approve",
            post_with(tools::set_tool_global_approval, |op| {
                op.description("Set global auto-approve for a tool")
                    .id("Mcp.setToolGlobalApproval")
                    .tag("mcp")
                    .response::<200, Json<serde_json::Value>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        .api_route(
            "/servers/{server_id}/tools/{tool_name}/approve",
            delete_with(tools::remove_tool_global_approval, |op| {
                op.description("Remove global auto-approve for a tool")
                    .id("Mcp.removeToolGlobalApproval")
                    .tag("mcp")
                    .response::<200, Json<serde_json::Value>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        .api_route(
            "/tools/execute",
            post_with(execution::execute_tool, |op| {
                op.description("Execute MCP tool")
                    .id("Mcp.executeTool")
                    .tag("mcp")
                    .response::<200, Json<ToolExecutionResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        // Execution logs
        .api_route(
            "/execution/logs",
            get_with(execution::list_user_execution_logs, |op| {
                op.description("List user's execution logs")
                    .id("Mcp.listExecutionLogs")
                    .tag("mcp")
                    .response::<200, Json<execution::ListExecutionLogsResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        .api_route(
            "/execution/logs/{id}",
            get_with(execution::get_execution_log, |op| {
                op.description("Get execution log by ID")
                    .id("Mcp.getExecutionLog")
                    .tag("mcp")
                    .response::<200, Json<crate::database::models::mcp_tool::MCPExecutionLog>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        .api_route(
            "/execution/logs/{id}/cancel",
            post_with(execution::cancel_execution, |op| {
                op.description("Cancel execution")
                    .id("Mcp.cancelExecution")
                    .tag("mcp")
                    .response::<200, Json<serde_json::Value>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        .api_route(
            "/threads/{thread_id}/execution/logs",
            get_with(execution::list_thread_execution_logs, |op| {
                op.description("List execution logs for thread")
                    .id("Mcp.listThreadExecutionLogs")
                    .tag("mcp")
                    .response::<200, Json<Vec<crate::database::models::mcp_tool::MCPExecutionLog>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        // User server assignments
        .api_route(
            "/user/assigned-servers",
            get_with(servers::get_user_assigned_servers, |op| {
                op.description("Get servers assigned to current user through groups")
                    .id("Mcp.getUserAssignedServers")
                    .tag("mcp")
                    .response::<200, Json<Vec<uuid::Uuid>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_servers_read_middleware,
            )),
        )
        // Conversation tool approvals
        .api_route(
            "/approvals/conversations/{conversation_id}",
            get_with(approvals::list_conversation_approvals, |op| {
                op.description("List approvals for specific conversation")
                    .id("Mcp.listConversationApprovals")
                    .tag("mcp")
                    .response::<200, Json<Vec<crate::database::models::mcp_tool::ToolApprovalResponse>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        .api_route(
            "/approvals/conversations/{conversation_id}",
            post_with(approvals::create_conversation_approval, |op| {
                op.description("Create or update approval for specific conversation")
                    .id("Mcp.createConversationApproval")
                    .tag("mcp")
                    .response::<201, Json<crate::database::models::mcp_tool::ToolApprovalResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        .api_route(
            "/approvals/conversations/{conversation_id}/tool",
            delete_with(approvals::delete_conversation_approval, |op| {
                op.description("Delete conversation approval for specific tool")
                    .id("Mcp.deleteConversationApproval")
                    .tag("mcp")
                    .response::<200, Json<approvals::SimpleResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        .api_route(
            "/approvals/conversations/{conversation_id}/check",
            get_with(approvals::check_conversation_approval, |op| {
                op.description("Check if tool is approved for conversation")
                    .id("Mcp.checkConversationApproval")
                    .tag("mcp")
                    .response::<200, Json<approvals::ApprovalCheckResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        // Tool approval management endpoints
        .api_route(
            "/approvals/{approval_id}",
            put_with(approvals::update_tool_approval, |op| {
                op.description("Update existing tool approval")
                    .id("Mcp.updateToolApproval")
                    .tag("mcp")
                    .response::<200, Json<crate::database::models::mcp_tool::ToolApprovalResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
        .api_route(
            "/servers/{server_id}/tools/{tool_name}/global-approval",
            get_with(approvals::get_global_tool_approval, |op| {
                op.description("Get global tool approval for specific server/tool")
                    .id("Mcp.getGlobalToolApproval")
                    .tag("mcp")
                    .response::<200, Json<crate::database::models::mcp_tool::ToolApprovalResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_read_middleware,
            )),
        )
        // Admin/maintenance endpoints
        .api_route(
            "/approvals/clean-expired",
            post_with(approvals::clean_expired_approvals, |op| {
                op.description("Clean expired tool approvals (admin)")
                    .id("Mcp.cleanExpiredApprovals")
                    .tag("mcp")
                    .response::<200, Json<serde_json::Value>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_tools_execute_middleware,
            )),
        )
}