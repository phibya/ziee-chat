//! Internal MCP Server for RAG instances
//!
//! This module provides a unified MCP server that exposes all RAG instances
//! as individual tools. The server runs internally on a random port and
//! enforces user-based access control.

pub mod global;
pub mod server;
pub mod startup;

// Re-export main functionality
pub use global::{get_rag_mcp_port, get_rag_mcp_url, set_rag_mcp_port};
pub use server::UnifiedRagMcpServer;
pub use startup::start_rag_mcp_server;
