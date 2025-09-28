pub mod server_manager;
pub mod transports;
pub mod protocol;
pub mod logging;

// Re-export main functionality
pub use server_manager::{
    start_mcp_server,
    stop_mcp_server,
    verify_mcp_server_running,
    reconcile_mcp_server_states,
    shutdown_all_mcp_servers,
    MCPServerStartResult,
};

pub use server_manager::auto_restart::{
  start_auto_restart_task,
  MCPAutoRestartConfig,
};