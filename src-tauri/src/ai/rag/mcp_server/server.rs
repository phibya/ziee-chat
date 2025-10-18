//! Internal RAG MCP Server
//!
//! This module provides a unified MCP server for all RAG instances.
//! Each instance is exposed as a separate tool (rag_query_{instance_id}).
//! Simple JSON-RPC 2.0 implementation without external dependencies.

use axum::{
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;


// ============================================
// JSON-RPC 2.0 Types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// ============================================
// MCP-specific Types
// ============================================

#[derive(Debug, Serialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}


// ============================================
// Unified RAG MCP Server
// ============================================

pub struct UnifiedRagMcpServer;

impl UnifiedRagMcpServer {
    /// Create Axum router for the MCP server
    pub fn router() -> Router {
        Router::new().route("/mcp", post(Self::handle_mcp_request))
    }

    /// Main MCP request handler
    async fn handle_mcp_request(Json(request): Json<JsonRpcRequest>) -> Response {
        let response = match request.method.as_str() {
            "initialize" => Self::handle_initialize(&request),
            "tools/list" => Self::handle_list_tools(&request).await,
            "tools/call" => Self::handle_call_tool(&request).await,
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: "Method not found".to_string(),
                    data: None,
                }),
            },
        };

        Json(response).into_response()
    }

    /// Handle MCP initialize request
    fn handle_initialize(request: &JsonRpcRequest) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "rag-mcp-server",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {}
                }
            })),
            error: None,
        }
    }

    /// List all available RAG tools (from engine tool definitions)
    async fn handle_list_tools(request: &JsonRpcRequest) -> JsonRpcResponse {
        tracing::debug!("Listing all RAG tools");

        // Get all enabled and active RAG instances from database
        let pool = match crate::database::get_database_pool() {
            Ok(p) => p,
            Err(e) => {
                return Self::error_response(request.id.clone(), -32603, &format!("Database error: {}", e));
            }
        };

        let instances = match sqlx::query!(
            r#"SELECT id, display_name, description, engine_type FROM rag_instances WHERE enabled = true AND is_active = true"#
        )
        .fetch_all(pool.as_ref())
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                return Self::error_response(request.id.clone(), -32603, &format!("Failed to list instances: {}", e));
            }
        };

        let mut all_tools = Vec::new();

        for instance_row in instances {
            let instance_id = instance_row.id;

            // Create engine to get its tool definitions
            use crate::ai::rag::engines::RAGEngineFactory;
            let engine = match RAGEngineFactory::create_engine(instance_id).await {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Failed to create engine for instance {}: {}", instance_id, e);
                    continue; // Skip on error
                }
            };

            let engine_tools = engine.get_tools();

            // Convert engine tools to MCP tools
            // Tool name format: rag_{instance_id_no_dashes}_{tool_name}
            // Remove dashes from UUID so we can easily split by underscore
            let instance_id_str = instance_id.to_string().replace("-", "");

            for engine_tool in engine_tools {
                let tool_name = format!("rag_{}_{}", instance_id_str, engine_tool.name);

                // Build description: "RAG: {instance_description} - {tool_description}"
                let rag_desc = instance_row.description.as_deref().unwrap_or(&instance_row.display_name);
                let description = format!(
                    "RAG: {} - {}",
                    rag_desc,
                    engine_tool.description
                );

                all_tools.push(McpTool {
                    name: tool_name,
                    description,
                    input_schema: engine_tool.input_schema,
                });
            }
        }

        tracing::info!("Listed {} RAG tools", all_tools.len());

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(json!({
                "tools": all_tools
            })),
            error: None,
        }
    }

    /// Execute tool call
    async fn handle_call_tool(request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match &request.params {
            Some(p) => p,
            None => return Self::error_response(request.id.clone(), -32602, "Missing params"),
        };

        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => return Self::error_response(request.id.clone(), -32602, "Missing tool name"),
        };

        let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

        tracing::info!("RAG tool call: {}", tool_name);

        // Parse tool name: "rag_{instance_id_no_dashes}_{tool_name}"
        // Example: "rag_abc123def456_query" -> parts = ["rag", "abc123def456", "query"]
        if !tool_name.starts_with("rag_") {
            return Self::error_response(request.id.clone(), -32602, "Invalid RAG tool name");
        }

        let parts: Vec<&str> = tool_name.split('_').collect();
        if parts.len() < 3 {
            return Self::error_response(request.id.clone(), -32602, "Invalid RAG tool name format");
        }

        // parts[0] = "rag", parts[1] = instance_id (no dashes), parts[2..] = tool_name
        let instance_id_str = parts[1];
        let engine_tool_name = parts[2..].join("_");

        // Parse UUID by adding dashes back in standard format
        // UUID format: 8-4-4-4-12 characters
        let uuid_with_dashes = if instance_id_str.len() == 32 {
            format!(
                "{}-{}-{}-{}-{}",
                &instance_id_str[0..8],
                &instance_id_str[8..12],
                &instance_id_str[12..16],
                &instance_id_str[16..20],
                &instance_id_str[20..32]
            )
        } else {
            return Self::error_response(request.id.clone(), -32602, "Invalid instance ID length");
        };

        let instance_id = match Uuid::parse_str(&uuid_with_dashes) {
            Ok(id) => id,
            Err(_) => return Self::error_response(request.id.clone(), -32602, "Invalid instance ID"),
        };

        // Create engine and execute tool
        use crate::ai::rag::engines::{RAGEngineFactory, traits::RAGToolCall};
        let engine = match RAGEngineFactory::create_engine(instance_id).await {
            Ok(e) => e,
            Err(e) => return Self::error_response(request.id.clone(), -32603, &format!("Failed to create engine: {}", e)),
        };

        // Execute the specific tool requested by name
        let call = RAGToolCall {
            tool_name: engine_tool_name,
            arguments,
        };

        let result = match engine.execute_tool(call).await {
            Ok(r) => r,
            Err(e) => return Self::error_response(request.id.clone(), -32603, &format!("Tool execution failed: {}", e)),
        };

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(result.result),
            error: None,
        }
    }

    /// Helper to create error response
    fn error_response(id: Option<Value>, code: i32, message: &str) -> JsonRpcResponse {
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }

}
