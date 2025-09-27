use async_trait::async_trait;
use std::process::{Command, Stdio};
use std::path::PathBuf;
use crate::database::models::mcp_server::MCPServer;
use crate::utils::resource_paths::ResourcePaths;
use super::{MCPTransport, MCPConnectionInfo};

pub struct MCPStdioTransport {
    server: MCPServer,
}

impl MCPStdioTransport {
    pub fn new(server: &MCPServer) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            server: server.clone(),
        })
    }
}

/// Resolve runtime commands to bundled executables if available
pub fn resolve_command(command: &str, args: &[String]) -> (PathBuf, Vec<String>) {
    match command {
        // npx -> bun x (Node.js package runner)
        "npx" => {
            if let Some(bun_path) = ResourcePaths::find_executable_binary("bun") {
                let mut new_args = vec!["x".to_string()];
                new_args.extend_from_slice(args);
                (bun_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },

        // node -> bun (if available)
        "node" => {
            if let Some(bun_path) = ResourcePaths::find_executable_binary("bun") {
                (bun_path, args.to_vec())
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },

        // npm -> bun (if available)
        "npm" => {
            if let Some(bun_path) = ResourcePaths::find_executable_binary("bun") {
                (bun_path, args.to_vec())
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },

        // pip -> uv pip
        "pip" | "pip3" => {
            if let Some(uv_path) = ResourcePaths::find_executable_binary("uv") {
                let mut new_args = vec!["pip".to_string()];
                new_args.extend_from_slice(args);
                (uv_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },

        // uvx -> uv tool run
        "uvx" => {
            if let Some(uv_path) = ResourcePaths::find_executable_binary("uv") {
                let mut new_args = vec!["tool".to_string(), "run".to_string()];
                new_args.extend_from_slice(args);
                (uv_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },

        // python -> uv run python (if available)
        "python" | "python3" => {
            if let Some(uv_path) = ResourcePaths::find_executable_binary("uv") {
                let mut new_args = vec!["run".to_string(), "python".to_string()];
                new_args.extend_from_slice(args);
                (uv_path, new_args)
            } else {
                (PathBuf::from(command), args.to_vec())
            }
        },

        // Everything else stays the same
        _ => (PathBuf::from(command), args.to_vec())
    }
}

#[async_trait]
impl MCPTransport for MCPStdioTransport {
    async fn start(&self) -> Result<MCPConnectionInfo, Box<dyn std::error::Error + Send + Sync>> {
        let command = self.server.command.as_ref()
            .ok_or("Command is required for stdio transport")?;

        let args: Vec<String> = serde_json::from_value(self.server.args.clone())
            .unwrap_or_default();

        // Resolve runtime command to bundled executable if available
        let (resolved_command, resolved_args) = resolve_command(command, &args);
        let mut cmd = Command::new(resolved_command);
        cmd.args(resolved_args)
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