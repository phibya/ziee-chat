use crate::api::mcp::{servers, tools, execution};
use crate::database::models::mcp_server::MCPServer;
use crate::database::models::user_group_mcp_server::GroupServerAssignmentResponse;
use servers::GroupAssignmentResponse;
use aide::axum::{
    routing::{delete_with, get_with, post_with},
    ApiRouter,
};
use axum::{middleware, Json};
use uuid::Uuid;

/// Create admin MCP routes
pub fn admin_mcp_routes() -> ApiRouter {
    ApiRouter::new()
        // System server management (admin only)
        .api_route(
            "/system-servers",
            get_with(servers::list_system_servers, |op| {
                op.description("List all system MCP servers (admin only)")
                    .id("AdminMcp.listSystemServers")
                    .tag("mcp-admin")
                    .response::<200, Json<servers::ListServersResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
        .api_route(
            "/system-servers",
            post_with(servers::create_system_server, |op| {
                op.description("Create new system MCP server (admin only)")
                    .id("AdminMcp.createSystemServer")
                    .tag("mcp-admin")
                    .response::<201, Json<MCPServer>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_create_middleware,
            )),
        )
        // Group assignment (admin only)
        .api_route(
            "/groups/{group_id}/servers",
            get_with(servers::get_group_servers, |op| {
                op.description("Get MCP servers assigned to group (admin only)")
                    .id("AdminMcp.getGroupServers")
                    .tag("mcp-admin")
                    .response::<200, Json<Vec<Uuid>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
        .api_route(
            "/groups/{group_id}/servers",
            post_with(servers::assign_servers_to_group, |op| {
                op.description("Assign MCP servers to group (admin only)")
                    .id("AdminMcp.assignServersToGroup")
                    .tag("mcp-admin")
                    .response::<200, Json<GroupAssignmentResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_edit_middleware,
            )),
        )
        .api_route(
            "/groups/{group_id}/servers/{server_id}",
            delete_with(servers::remove_server_from_group, |op| {
                op.description("Remove server from group (admin only)")
                    .id("AdminMcp.removeServerFromGroup")
                    .tag("mcp-admin")
                    .response::<204, ()>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_edit_middleware,
            )),
        )
        // Tool statistics (admin only)
        .api_route(
            "/tools/statistics",
            get_with(tools::get_tool_statistics, |op| {
                op.description("Get MCP tool usage statistics (admin only)")
                    .id("AdminMcp.getToolStatistics")
                    .tag("mcp-admin")
                    .response::<200, Json<Vec<(String, String, i32, i64)>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
        // Execution statistics and logs (admin only)
        .api_route(
            "/execution/statistics",
            get_with(execution::get_execution_statistics, |op| {
                op.description("Get MCP execution statistics (admin only)")
                    .id("AdminMcp.getExecutionStatistics")
                    .tag("mcp-admin")
                    .response::<200, Json<serde_json::Value>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
        .api_route(
            "/execution/logs",
            get_with(execution::list_all_execution_logs, |op| {
                op.description("List all execution logs (admin only)")
                    .id("AdminMcp.listAllExecutionLogs")
                    .tag("mcp-admin")
                    .response::<200, Json<execution::ListExecutionLogsResponse>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
        // Group-server assignment management
        .api_route(
            "/assignments",
            get_with(servers::list_all_group_assignments, |op| {
                op.description("List all group-server assignments (admin only)")
                    .id("AdminMcp.listAllGroupAssignments")
                    .tag("mcp-admin")
                    .response::<200, Json<Vec<GroupServerAssignmentResponse>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
        // Server access management
        .api_route(
            "/servers/{server_id}/groups",
            get_with(servers::get_server_access_groups, |op| {
                op.description("Get groups that have access to a specific server (admin only)")
                    .id("AdminMcp.getServerAccessGroups")
                    .tag("mcp-admin")
                    .response::<200, Json<Vec<uuid::Uuid>>>()
            })
            .layer(middleware::from_fn(
                crate::api::middleware::permissions::mcp_admin_servers_read_middleware,
            )),
        )
}