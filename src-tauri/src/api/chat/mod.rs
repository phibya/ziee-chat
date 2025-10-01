//! Chat API module
//!
//! This module provides SSE streaming chat functionality with AI provider integration,
//! including tool approval workflows and message branching.
//!
//! ## Module Structure
//!
//! - `types`: SSE event types and request/response structures
//! - `helpers`: Helper functions (error handling, title generation)
//! - `tool_handling`: Tool approval and execution logic
//! - `streaming`: Core streaming logic for AI responses
//! - `handlers`: Public API handlers for chat operations
//!
//! ## Public API
//!
//! This module re-exports the public API handlers and types that are used by the router.

mod handlers;
mod helpers;
mod streaming;
mod tool_handling;
mod types;

// Re-export public items used by router
pub use handlers::{
    edit_message_stream, get_conversation_messages_by_branch, get_message_branches,
    send_message_stream,
};

// Re-export types used in OpenAPI generation and router
pub use types::{ChatMessageRequest, SSEChatStreamEvent};
