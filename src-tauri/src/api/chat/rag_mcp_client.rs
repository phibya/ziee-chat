//! RAG MCP Client for Chat Integration
//!
//! This module provides functions to call the internal RAG MCP server
//! for getting tools and executing tool calls from chat.

use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::database::models::chat::ToolDefinition;
use crate::database::queries::rag_instances::validate_rag_instance_access;

#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: i32,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: i32,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct McpTool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: serde_json::Value,
}

/// Get RAG tool definitions from RAG MCP server, filtered by enabled RAG IDs
pub async fn get_rag_tools_from_mcp(
    user_id: Uuid,
    enabled_rag_ids: &[Uuid],
) -> Result<Vec<ToolDefinition>, Box<dyn std::error::Error + Send + Sync>> {
    println!("DEBUG: get_rag_tools_from_mcp called with user_id={}, enabled_rag_ids={:?}", user_id, enabled_rag_ids);

    // First, verify user has access to all enabled RAG instances
    for rag_id in enabled_rag_ids {
        println!("DEBUG: Checking access for RAG instance {}", rag_id);
        let has_access = validate_rag_instance_access(user_id, *rag_id, false).await?;
        if !has_access {
            println!("ERROR: User {} does not have access to RAG instance {}", user_id, rag_id);
            return Err(format!("User does not have access to RAG instance: {}", rag_id).into());
        }
        println!("DEBUG: Access granted for RAG instance {}", rag_id);
    }

    // Call internal RAG MCP server's list_tools method
    // This returns ALL enabled RAG instance tools
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 1,
        method: "tools/list".to_string(),
        params: None,
    };

    println!("DEBUG: Calling RAG MCP server for tools/list");
    let response = call_rag_mcp_server(request).await?;
    println!("DEBUG: Got response from RAG MCP server");

    if let Some(error) = response.error {
        println!("ERROR: RAG MCP server returned error: {:?}", error);
        return Err(format!("RAG MCP server error: {:?}", error).into());
    }

    let result = response.result.ok_or("No result from MCP server")?;
    println!("DEBUG: Response result: {:?}", result);

    let tools: Vec<McpTool> = serde_json::from_value(
        result.get("tools").ok_or("No tools in response")?.clone()
    )?;
    println!("DEBUG: Parsed {} tools from MCP server", tools.len());

    // Filter tools to only include those from enabled RAG instances
    // Tool name format: rag_{instance_id_no_dashes}_{tool_name}
    let enabled_rag_ids_no_dashes: Vec<String> = enabled_rag_ids
        .iter()
        .map(|id| id.to_string().replace("-", ""))
        .collect();
    println!("DEBUG: Filtering tools by enabled RAG IDs (no dashes): {:?}", enabled_rag_ids_no_dashes);

    let filtered_tools: Vec<ToolDefinition> = tools
        .into_iter()
        .filter(|tool| {
            // Extract instance_id from tool name
            if let Some(parts) = tool.name.strip_prefix("rag_") {
                let parts: Vec<&str> = parts.split('_').collect();
                if !parts.is_empty() {
                    let instance_id_no_dashes = parts[0];
                    let matches = enabled_rag_ids_no_dashes.contains(&instance_id_no_dashes.to_string());
                    println!("DEBUG: Tool '{}' instance_id={} matches={}", tool.name, instance_id_no_dashes, matches);
                    return matches;
                }
            }
            println!("DEBUG: Tool '{}' does not match rag_ prefix pattern", tool.name);
            false
        })
        .map(|t| ToolDefinition {
            name: t.name,
            description: Some(t.description),
            input_schema: t.input_schema,
        })
        .collect();

    println!("DEBUG: Filtered to {} tools", filtered_tools.len());
    Ok(filtered_tools)
}

/// Execute RAG tool via RAG MCP server
pub async fn execute_rag_tool_via_mcp(
    tool_name: String,
    arguments: serde_json::Value,
) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: 2,
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": tool_name,
            "arguments": arguments,
        })),
    };

    let response = call_rag_mcp_server(request).await?;

    if let Some(error) = response.error {
        return Err(format!("RAG MCP tool execution error: {:?}", error).into());
    }

    response.result.ok_or("No result from MCP server".into())
}

/// Internal function to call RAG MCP server
async fn call_rag_mcp_server(
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse, Box<dyn std::error::Error + Send + Sync>> {
    // Construct internal MCP request
    use reqwest::Client;

    // Get the RAG MCP server URL from global state
    use crate::ai::rag::mcp_server::global::get_rag_mcp_url;
    let mcp_url = get_rag_mcp_url()
        .ok_or("RAG MCP server not started or port not available")?;

    println!("DEBUG: Calling RAG MCP server at {} with method {}", mcp_url, request.method);
    println!("DEBUG: Request: {:?}", serde_json::to_string(&request).unwrap_or_else(|_| "failed to serialize".to_string()));

    let client = Client::new();
    let response = client
        .post(&mcp_url)
        .json(&request)
        .send()
        .await?;

    let status = response.status();
    println!("DEBUG: Response status: {}", status);

    let response_text = response.text().await?;
    println!("DEBUG: Response body: {}", response_text);

    let json_response: JsonRpcResponse = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse JSON response: {}. Response body: {}", e, response_text))?;

    Ok(json_response)
}
