use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde_json;

use super::{ModelRegistry, ProxyError, log_request};

#[derive(Debug)]
pub struct RequestRouter {
    registry: Arc<RwLock<ModelRegistry>>,
    client: reqwest::Client,
}

impl RequestRouter {
    pub fn new(registry: Arc<RwLock<ModelRegistry>>) -> Self {
        Self {
            registry,
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn route_chat_request(&self, mut request: serde_json::Value) -> Result<serde_json::Value, ProxyError> {
        // 1. Extract model identifier from request or use default
        let (model_id, original_identifier) = self.extract_or_default_model_id(&mut request).await?;
        
        // 2. Log the request
        let registry = self.registry.read().await;
        let display_name = registry.get_model_display_name(&model_id).await;
        log_request("POST", "/chat/completions", "proxy", Some(&display_name));
        
        // 3. Get base URL for the model (handles both local and remote)
        let base_url = registry.get_model_base_url(&original_identifier).await?;
        drop(registry); // Release the lock
        
        // 4. Forward request to model server/provider
        let url = format!("{}/v1/chat/completions", base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProxyError::ServerUnreachable(e.to_string()))?;
            
        // 5. Return response as-is
        let response_json = response.json().await
            .map_err(|e| ProxyError::InvalidResponse(e.to_string()))?;
            
        Ok(response_json)
    }
    
    pub async fn route_streaming_request(&self, mut request: serde_json::Value) -> Result<reqwest::Response, ProxyError> {
        // Similar to route_chat_request but return raw response for streaming
        let (model_id, original_identifier) = self.extract_or_default_model_id(&mut request).await?;
        
        let registry = self.registry.read().await;
        let display_name = registry.get_model_display_name(&model_id).await;
        log_request("POST", "/chat/completions", "proxy", Some(&display_name));
        
        let base_url = registry.get_model_base_url(&original_identifier).await?;
        drop(registry);
        
        let url = format!("{}/v1/chat/completions", base_url);
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| ProxyError::ServerUnreachable(e.to_string()))?;
            
        Ok(response)
    }
    
    pub async fn handle_models_request(&self) -> Result<serde_json::Value, ProxyError> {
        // 1. Get all enabled models from registry
        let registry = self.registry.read().await;
        let enabled_models = registry.list_enabled_models();
        
        // 2. Create model list with aliases
        let mut models_data = Vec::new();
        for model_entry in enabled_models {
            let display_name = if let Some(alias) = &model_entry.alias_id {
                alias.clone()
            } else {
                model_entry.model_id.to_string()
            };
            
            models_data.push(serde_json::json!({
                "id": display_name,
                "object": "model",
                "created": chrono::Utc::now().timestamp(),
                "owned_by": "api-proxy-server"
            }));
        }
        
        // 3. Return aggregated models list with aliases
        Ok(serde_json::json!({
            "object": "list",
            "data": models_data
        }))
    }
    
    async fn extract_or_default_model_id(&self, request: &mut serde_json::Value) -> Result<(Uuid, String), ProxyError> {
        // Try to get model from request
        let model_str_opt = request.get("model").and_then(|m| m.as_str()).map(|s| s.to_string());
        
        if let Some(model_str) = model_str_opt {
            // Resolve model identifier (UUID or alias) to actual UUID
            let registry = self.registry.read().await;
            let model_id = registry.resolve_model_identifier(&model_str).await?;
            
            // Update request with actual UUID for backend compatibility
            request["model"] = serde_json::json!(model_id.to_string());
            Ok((model_id, model_str))
        } else {
            // No model specified, use default
            let registry = self.registry.read().await;
            let default_model = registry.get_default_model().await
                .ok_or(ProxyError::NoDefaultModel)?;
            
            // Add default model to request
            request["model"] = serde_json::json!(default_model.to_string());
            Ok((default_model, default_model.to_string()))
        }
    }
    
    async fn get_models_from_server(&self, base_url: &str) -> Result<Vec<serde_json::Value>, ProxyError> {
        let url = format!("{}/v1/models", base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| ProxyError::ServerUnreachable(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(ProxyError::ServerUnreachable(format!("Status: {}", response.status())));
        }
        
        let models_response: serde_json::Value = response.json().await
            .map_err(|e| ProxyError::InvalidResponse(e.to_string()))?;
        
        let models_data = models_response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or(ProxyError::InvalidResponse("No data array in models response".to_string()))?;
        
        Ok(models_data.clone())
    }
}