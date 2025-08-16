use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde_json;

use super::{ModelRegistry, ProxyError, log_request, HttpForwardingProvider};
use crate::database::queries::models;

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
            
            // Get the actual model name from database for provider compatibility
            let model = models::get_model_by_id(model_id).await
                .map_err(|e| ProxyError::DatabaseError(e.to_string()))?
                .ok_or(ProxyError::ModelNotFound(model_id.to_string()))?;
            
            // Update request with actual model name for provider compatibility
            request["model"] = serde_json::json!(model.name);
            Ok((model_id, model_str))
        } else {
            // No model specified, use default
            let registry = self.registry.read().await;
            let default_model = registry.get_default_model().await
                .ok_or(ProxyError::NoDefaultModel)?;
            
            // Get the actual model name from database for provider compatibility
            let model = models::get_model_by_id(default_model).await
                .map_err(|e| ProxyError::DatabaseError(e.to_string()))?
                .ok_or(ProxyError::ModelNotFound(default_model.to_string()))?;
            
            // Add default model name to request
            request["model"] = serde_json::json!(model.name);
            Ok((default_model, default_model.to_string()))
        }
    }
    
    /// Forward chat completion request to appropriate provider
    pub async fn forward_chat_request(&self, mut request: serde_json::Value) -> Result<reqwest::Response, ProxyError> {
        // 1. Extract model identifier and resolve to UUID
        let (model_id, _) = self.extract_or_default_model_id(&mut request).await?;
        
        // 2. Log the request
        let registry = self.registry.read().await;
        let display_name = registry.get_model_display_name(&model_id).await;
        log_request("POST", "/chat/completions", "proxy", Some(&display_name));
        
        // 3. Get forwarding provider for the model
        let provider = registry.get_forwarding_provider(&model_id).await?;
        drop(registry); // Release the lock early
        
        // 4. Forward request using provider's implementation
        let response = provider.forward_request(request).await
            .map_err(|e| ProxyError::ServerUnreachable(e.to_string()))?;
            
        Ok(response)
    }
}