use crate::database::queries::{mcp_servers, mcp_execution_logs};
use crate::mcp::server_manager::{start_mcp_server, reconcile_mcp_server_states};
use crate::mcp::{start_auto_restart_task, MCPAutoRestartConfig};
use crate::database::models::mcp_server::MCPServerStatus;
use tracing::{info, warn, error};

/// Initialize MCP servers that were active in the previous run
pub async fn initialize_mcp() -> Result<(), String> {
    info!("Initializing MCP servers...");

    // Clean up old execution logs (older than 30 days)
    match mcp_execution_logs::cleanup_old_execution_logs(30).await {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                info!("Cleaned up {} old MCP execution logs", deleted_count);
            }
        }
        Err(e) => {
            warn!("Failed to cleanup old execution logs: {}", e);
        }
    }

    // Get all enabled servers and reconcile their states
    match mcp_servers::get_all_enabled_mcp_servers().await {
        Ok(servers) => {
            info!("Found {} enabled MCP servers", servers.len());

            // Reconcile server states first
            if let Err(e) = reconcile_mcp_server_states().await {
                warn!("Failed to reconcile MCP server states: {}", e);
            }

            let mut started_count = 0;
            let mut failed_count = 0;

            // Start servers that should be running
            for server in servers {
                if server.status == MCPServerStatus::Running {
                    info!("Attempting to restart MCP server: {} ({})", server.name, server.id);

                    match start_mcp_server(&server.id).await {
                        Ok(_) => {
                            info!("Successfully restarted MCP server: {}", server.name);
                            started_count += 1;
                        }
                        Err(e) => {
                            error!("Failed to restart MCP server {}: {}", server.name, e);
                            failed_count += 1;

                            // Update server status to stopped since restart failed
                            if let Err(update_err) = mcp_servers::update_server_status(
                                server.id,
                                MCPServerStatus::Stopped,
                                false, // is_active
                                None,  // process_id
                                None,  // port
                            ).await {
                                warn!("Failed to update server status for {}: {}", server.name, update_err);
                            }
                        }
                    }
                }
            }

            info!(
                "MCP server initialization complete: {} started, {} failed",
                started_count, failed_count
            );
        }
        Err(e) => {
            warn!("Failed to retrieve enabled MCP servers: {}", e);
        }
    }

    // Start auto-restart task for system servers
    let restart_config = MCPAutoRestartConfig::default();
    start_auto_restart_task(restart_config);
    info!("MCP server auto-restart task started");

    Ok(())
}

/// Cleanup MCP servers on application shutdown
pub async fn cleanup_mcp() {
    info!("Cleaning up MCP servers...");

    // Use the shutdown_all function which handles cleanup properly
    match crate::mcp::server_manager::shutdown_all_mcp_servers().await {
        Ok(_) => {
            info!("Successfully shut down all MCP servers");
        }
        Err(e) => {
            error!("Failed to shutdown MCP servers: {}", e);
        }
    }

    info!("MCP server cleanup complete");
}