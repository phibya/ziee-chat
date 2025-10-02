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

use crate::ai::rag::{RAGQuery, QueryMode};
use crate::database::queries::rag_instances::get_rag_instance_by_id;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RAGQueryArguments {
    pub text: String,
    #[serde(default = "default_query_mode")]
    pub mode: String,
}

fn default_query_mode() -> String {
    "naive".to_string()
}

impl RAGQueryArguments {
    fn to_query_mode(&self) -> QueryMode {
        match self.mode.as_str() {
            "local" => QueryMode::Local,
            "global" => QueryMode::Global,
            "hybrid" => QueryMode::Hybrid,
            "mix" => QueryMode::Mix,
            "naive" => QueryMode::Naive,
            "bypass" => QueryMode::Bypass,
            _ => QueryMode::Naive,
        }
    }
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

    /// List all available RAG tools
    async fn handle_list_tools(request: &JsonRpcRequest) -> JsonRpcResponse {
        tracing::debug!("Listing all RAG tools");

        // Get all RAG instances from database
        let pool = match crate::database::get_database_pool() {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("Database error: {}", e),
                        data: None,
                    }),
                };
            }
        };

        let instances = match sqlx::query!(
            r#"SELECT id, display_name, description FROM rag_instances WHERE enabled = true"#
        )
        .fetch_all(pool.as_ref())
        .await
        {
            Ok(rows) => rows,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32603,
                        message: format!("Failed to list instances: {}", e),
                        data: None,
                    }),
                };
            }
        };

        let tools: Vec<McpTool> = instances
            .into_iter()
            .map(|instance| {
                let tool_name = format!("rag_query_{}", instance.id);
                let description = format!(
                    "Query RAG instance '{}': {}",
                    instance.display_name,
                    instance.description.as_deref().unwrap_or("No description")
                );

                McpTool {
                    name: tool_name,
                    description,
                    input_schema: json!({
                        "type": "object",
                        "properties": {
                            "text": {
                                "type": "string",
                                "description": "The query text to search for"
                            },
                            "mode": {
                                "type": "string",
                                "enum": ["naive", "local", "global", "hybrid", "mix", "bypass"],
                                "default": "naive",
                                "description": "Query mode (default: naive)"
                            }
                        },
                        "required": ["text"]
                    }),
                }
            })
            .collect();

        tracing::info!("Listed {} RAG tools", tools.len());

        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(json!({
                "tools": tools
            })),
            error: None,
        }
    }

    /// Execute tool call
    async fn handle_call_tool(request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match &request.params {
            Some(p) => p,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing params".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let tool_name = match params.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: "Missing tool name".to_string(),
                        data: None,
                    }),
                };
            }
        };

        let arguments: RAGQueryArguments =
            match serde_json::from_value(params.get("arguments").cloned().unwrap_or(json!({}))) {
                Ok(args) => args,
                Err(e) => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id.clone(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Invalid arguments: {}", e),
                            data: None,
                        }),
                    };
                }
            };

        tracing::info!("RAG tool call: {} with query '{}'", tool_name, arguments.text);

        // Extract instance_id from tool name
        let instance_id = match Self::parse_instance_id_from_tool(tool_name) {
            Ok(id) => id,
            Err(msg) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id.clone(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: msg,
                        data: None,
                    }),
                };
            }
        };

        // Execute the query
        match Self::execute_rag_query(instance_id, arguments).await {
            Ok(content) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(json!({
                    "content": [content]
                })),
                error: None,
            },
            Err(msg) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32603,
                    message: msg,
                    data: None,
                }),
            },
        }
    }

    /// Parse instance ID from tool name
    fn parse_instance_id_from_tool(tool_name: &str) -> Result<Uuid, String> {
        let prefix = "rag_query_";
        if !tool_name.starts_with(prefix) {
            return Err(format!("Invalid tool name: {}", tool_name));
        }

        let id_str = &tool_name[prefix.len()..];
        Uuid::parse_str(id_str)
            .map_err(|_| format!("Invalid instance ID in tool name: {}", tool_name))
    }

    /// Execute RAG query on the specified instance
    async fn execute_rag_query(
        instance_id: Uuid,
        arguments: RAGQueryArguments,
    ) -> Result<Value, String> {
        use crate::ai::rag::engines::simple_vector::RAGSimpleVectorEngine;
        use crate::ai::rag::engines::traits::RAGEngine;

        // Verify instance exists
        let instance = get_rag_instance_by_id(instance_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| format!("RAG instance not found: {}", instance_id))?;

        if !instance.enabled {
            return Err(format!("RAG instance is disabled: {}", instance_id));
        }

        let query = RAGQuery {
            text: arguments.text.clone(),
            mode: arguments.to_query_mode(),
        };

        // Create RAG engine
        let engine = RAGSimpleVectorEngine::new(instance_id)
            .await
            .map_err(|e| format!("Failed to create RAG engine: {}", e))?;

        // Execute the query through RAG engine
        let response = engine
            .query(query)
            .await
            .map_err(|e| format!("RAG query failed: {}", e))?;

        // Format response as MCP Content
        Ok(json!({
            "type": "text",
            "text": serde_json::to_string_pretty(&json!({
                "sources": response.sources,
                "mode_used": format!("{:?}", response.mode_used),
                "confidence_score": response.confidence_score,
                "processing_time_ms": response.processing_time_ms,
                "metadata": response.metadata,
            })).unwrap_or_default()
        }))
    }
}
