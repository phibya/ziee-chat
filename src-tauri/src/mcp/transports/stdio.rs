use async_trait::async_trait;
use std::process::{Command, Stdio};
use crate::database::models::mcp_server::MCPServer;
use super::{MCPTransport, MCPConnectionInfo};

pub struct StdioTransport {
    server: MCPServer,
}

impl StdioTransport {
    pub fn new(server: &MCPServer) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            server: server.clone(),
        })
    }
}

#[async_trait]
impl MCPTransport for StdioTransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let command = self.server.command.as_ref()
            .ok_or("Command is required for stdio transport")?;

        let args: Vec<String> = serde_json::from_value(self.server.args.clone())
            .unwrap_or_default();

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Always set the IS_ZIEE_MCP environment variable
        cmd.env("IS_ZIEE_MCP", "1");

        // Set custom environment variables
        if let Ok(env_vars) = serde_json::from_value::<serde_json::Map<String, serde_json::Value>>(self.server.environment_variables.clone()) {
            for (key, value) in env_vars {
                if let Some(val_str) = value.as_str() {
                    cmd.env(key, val_str);
                }
            }
        }

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn MCP server process: {}", e))?;

        let pid = child.id();

        Ok(MCPConnectionInfo {
            child: Some(child),
            pid: Some(pid),
            port: None,
        })
    }

    async fn stop(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Handled by server_manager
        Ok(())
    }

    async fn is_healthy(&self) -> bool {
        // For stdio, we check if process is still running
        // This is handled by the server manager's process verification
        true
    }
}