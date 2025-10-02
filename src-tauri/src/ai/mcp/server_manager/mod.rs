pub mod server_manager;
pub mod auto_restart;

// Re-export main functionality
pub use server_manager::{
    start_mcp_server,
    stop_mcp_server,
    verify_mcp_server_running,
    reconcile_mcp_server_states,
    shutdown_all_mcp_servers,
    get_server_process_info,
    get_running_server_ids,
    MCPServerStartResult,
};

pub use auto_restart::{
  start_auto_restart_task,
  MCPAutoRestartConfig,
};