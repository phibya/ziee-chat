use std::collections::HashMap;
use std::process::Child;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::database::models::mcp_server::MCPTransportType;
use crate::database::queries::mcp_servers;
use crate::mcp::transports::create_mcp_transport;
use crate::mcp::proxy::get_proxy_manager;

#[derive(Debug)]
struct MCPServerProcess {
    child: Option<Child>, // None for HTTP/SSE servers
    pid: Option<u32>,
    port: Option<u16>,
    transport_type: MCPTransportType,
    server_id: Uuid,
}

// Global registry to track running MCP servers
static MCP_SERVER_REGISTRY: std::sync::LazyLock<Arc<RwLock<HashMap<Uuid, MCPServerProcess>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

// Global mutex for all server starting operations
static GLOBAL_MCP_START_MUTEX: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MCPServerStartResult {
    Started {
        pid: Option<u32>,
        port: Option<u16>,
    },
    AlreadyRunning {
        pid: Option<u32>,
        port: Option<u16>,
    },
    Failed {
        error: String,
        log_path: String,
    },
}

/// Start an MCP server using appropriate transport
pub async fn start_mcp_server(
    server_id: &Uuid,
) -> Result<MCPServerStartResult, Box<dyn std::error::Error + Send + Sync>> {
    let _lock = GLOBAL_MCP_START_MUTEX.lock().await;

    // Load server from database
    let server = mcp_servers::get_mcp_server_by_id(*server_id)
        .await
        .map_err(|e| format!("Failed to load server: {}", e))?
        .ok_or_else(|| format!("Server {} not found", server_id))?;

    // Only check if already running for stdio transport (which spawns processes)
    if matches!(server.transport_type, MCPTransportType::Stdio) {
        if verify_mcp_server_running(&server).await {
            return Ok(MCPServerStartResult::AlreadyRunning {
                pid: server.process_id.map(|p| p as u32),
                port: server.port.map(|p| p as u16),
            });
        }
    }

    // Create transport based on server configuration
    let transport = create_mcp_transport(&server).await?;

    // Start the transport
    match transport.start().await {
        Ok(connection_info) => {
            // Register in registry
            if let Ok(mut registry) = MCP_SERVER_REGISTRY.write() {
                registry.insert(*server_id, MCPServerProcess {
                    child: connection_info.child,
                    pid: connection_info.pid,
                    port: connection_info.port,
                    transport_type: server.transport_type,
                    server_id: *server_id,
                });
            }

            // Update database
            mcp_servers::update_mcp_server_runtime_info(
                server_id,
                connection_info.pid.map(|p| p as i32),
                connection_info.port.map(|p| p as i32),
                "running".to_string(),
                true,
            ).await?;

            // For stdio servers, the proxy URL will be accessible at http://127.0.0.1:{port}/mcp
            // The database port field now stores the proxy port for internal reference

            // Update restart count if this is a restart operation
            let _ = mcp_servers::update_server_restart_count(
                *server_id,
                1 // increment by 1
            ).await;

            Ok(MCPServerStartResult::Started {
                pid: connection_info.pid,
                port: connection_info.port,
            })
        }
        Err(e) => {
            Ok(MCPServerStartResult::Failed {
                error: e.to_string(),
                log_path: create_server_log_path(server_id),
            })
        }
    }
}

/// Stop an MCP server
pub async fn stop_mcp_server(
    server_id: &Uuid,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Get from registry
    let server_process = {
        if let Ok(mut registry) = MCP_SERVER_REGISTRY.write() {
            registry.remove(server_id)
        } else {
            None
        }
    };

    if let Some(process) = server_process {
        // For stdio transport, stop the proxy instead of killing the process directly
        if matches!(process.transport_type, MCPTransportType::Stdio) {
            let proxy_manager = get_proxy_manager();
            let _ = proxy_manager.stop_proxy(server_id).await;
        } else {
            // Kill child process if exists (for other transports)
            if let Some(mut child) = process.child {
                let _ = child.kill();
                let _ = child.wait();
            }

            // For HTTP/SSE, we might need to send shutdown request
            if matches!(process.transport_type, MCPTransportType::Http | MCPTransportType::Sse) {
                // Send shutdown request to server if supported
                // This depends on the specific MCP server implementation
            }
        }
    }

    // Update database status
    mcp_servers::update_mcp_server_runtime_info(
        server_id,
        None,
        None,
        "stopped".to_string(),
        false,
    ).await?;

    Ok(())
}

/// Verify MCP server is running and responsive
pub async fn verify_mcp_server_running(
    server: &crate::database::models::mcp_server::MCPServer,
) -> bool {
    match server.transport_type {
        MCPTransportType::Stdio => {
            // For stdio servers, check if process is running
            if let Some(pid) = server.process_id {
                if !is_process_running(pid as u32) {
                    // Clean up stale database entry and registry
                    let _ = mcp_servers::update_mcp_server_runtime_info(
                        &server.id,
                        None,
                        None,
                        "stopped".to_string(),
                        false,
                    ).await;

                    // Remove from registry as well
                    if let Ok(mut registry) = MCP_SERVER_REGISTRY.write() {
                        registry.remove(&server.id);
                    }

                    return false;
                }
                true
            } else {
                false
            }
        }
        MCPTransportType::Http | MCPTransportType::Sse => {
            // For HTTP/SSE servers, make health check request
            if let Some(port) = server.port {
                verify_mcp_server_health(port as u16).await
            } else {
                false
            }
        }
    }
}

/// Health check for HTTP/SSE MCP servers
async fn verify_mcp_server_health(_port: u16) -> bool {
    // Implement MCP protocol health check
    // This could be a simple HTTP request or MCP ping message
    true // Placeholder
}

/// Auto-start MCP servers on app startup
pub async fn reconcile_mcp_server_states() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting MCP server state reconciliation...");

    // Get all enabled servers
    let servers = mcp_servers::get_all_enabled_mcp_servers().await?;

    for server in servers {
        // Verify if actually running
        if !verify_mcp_server_running(&server).await {
            // Server should be running but isn't, update database
            mcp_servers::update_mcp_server_runtime_info(
                &server.id,
                None,
                None,
                "stopped".to_string(),
                false,
            ).await?;
        }

        // Auto-start system servers marked as enabled
        if server.is_system && server.enabled {
            println!("Auto-starting system MCP server: {}", server.name);
            if let Err(e) = start_mcp_server(&server.id).await {
                eprintln!("Failed to auto-start system server {}: {}", server.name, e);
            }
        }
    }

    Ok(())
}

/// Shutdown all MCP servers on app exit
pub async fn shutdown_all_mcp_servers() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Shutting down all MCP servers...");

    // Get all running processes with their details for better logging
    let running_processes: Vec<(Uuid, Option<u32>, MCPTransportType)> = {
        if let Ok(registry) = MCP_SERVER_REGISTRY.read() {
            registry.iter().map(|(_, process)| {
                (process.server_id, process.pid, process.transport_type.clone())
            }).collect()
        } else {
            Vec::new()
        }
    };

    for (server_id, pid, transport_type) in running_processes {
        println!("Stopping MCP server {} (PID: {:?}, Transport: {:?})", server_id, pid, transport_type);
        if let Err(e) = stop_mcp_server(&server_id).await {
            eprintln!("Failed to stop MCP server {} (PID: {:?}): {}", server_id, pid, e);
        }
    }

    // Clear registry
    if let Ok(mut registry) = MCP_SERVER_REGISTRY.write() {
        registry.clear();
    }

    // Also shutdown all proxies to ensure clean shutdown
    let proxy_manager = get_proxy_manager();
    let _ = proxy_manager.shutdown_all_proxies().await;

    Ok(())
}

fn create_server_log_path(server_id: &Uuid) -> String {
    let log_dir = crate::get_app_data_dir().join("logs/mcp");
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).ok();
    }
    log_dir
        .join(format!("{}.log", server_id))
        .to_string_lossy()
        .to_string()
}

/// Get server process info from registry
pub fn get_server_process_info(server_id: &Uuid) -> Option<(Option<u32>, Option<u16>, MCPTransportType)> {
    if let Ok(registry) = MCP_SERVER_REGISTRY.read() {
        if let Some(process) = registry.get(server_id) {
            return Some((process.pid, process.port, process.transport_type.clone()));
        }
    }
    None
}

/// Get all running server IDs from registry
pub fn get_running_server_ids() -> Vec<Uuid> {
    if let Ok(registry) = MCP_SERVER_REGISTRY.read() {
        registry.keys().copied().collect()
    } else {
        Vec::new()
    }
}

fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::{Command, Stdio};

        // First check if the process exists
        let process_exists = match Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => status.success(),
            Err(_) => false,
        };

        if !process_exists {
            return false;
        }

        // Check if the process has IS_ZIEE_MCP=1 environment variable
        // First try Linux /proc method
        if let Ok(env_data) = std::fs::read_to_string(format!("/proc/{}/environ", pid)) {
            // Environment variables are null-separated in /proc/PID/environ
            return env_data.split('\0').any(|env_var| env_var == "IS_ZIEE_MCP=1");
        }

        // Fallback for macOS and other Unix systems using ps
        match Command::new("ps")
            .arg("-p")
            .arg(pid.to_string())
            .arg("-wwE")  // Show environment variables with full width
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => {
                let env_output = String::from_utf8_lossy(&output.stdout);
                // Check if IS_ZIEE_MCP=1 appears in the environment variables
                env_output.contains("IS_ZIEE_MCP=1")
            }
            Err(_) => {
                false
            }
        }
    }
    #[cfg(windows)]
    {
        use std::process::{Command, Stdio};

        // First check if the process exists
        let process_exists = match Command::new("tasklist")
            .arg("/FI")
            .arg(&format!("PID eq {}", pid))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
        {
            Ok(status) => status.success(),
            Err(_) => false,
        };

        if !process_exists {
            return false;
        }

        // Check if the process has IS_ZIEE_MCP=1 environment variable using PowerShell
        // PowerShell command to get environment variables of a specific process
        let ps_command = format!(
            "(Get-Process -Id {}).StartInfo.EnvironmentVariables | Out-String",
            pid
        );

        match Command::new("powershell")
            .arg("-NoProfile")
            .arg("-NonInteractive")
            .arg("-Command")
            .arg(&ps_command)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .output()
        {
            Ok(output) => {
                let env_output = String::from_utf8_lossy(&output.stdout);
                // Check if IS_ZIEE_MCP=1 appears in the environment variables
                env_output.contains("IS_ZIEE_MCP") && env_output.contains("1")
            }
            Err(_) => {
                // Alternative method using WMI if PowerShell fails
                let wmi_command = format!(
                    "wmic process where \"processid={}\" get commandline /format:list",
                    pid
                );

                match Command::new("cmd")
                    .arg("/C")
                    .arg(&wmi_command)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .output()
                {
                    Ok(output) => {
                        let cmd_output = String::from_utf8_lossy(&output.stdout);
                        // This is a fallback - we can't easily get env vars with WMI,
                        // so we fall back to basic process check
                        cmd_output.contains(&pid.to_string())
                    }
                    Err(_) => {
                        false
                    }
                }
            }
        }
    }
}