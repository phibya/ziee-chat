use std::collections::HashMap;
use uuid::Uuid;

use super::ProxyError;
use crate::database::models::api_proxy_server_model::ModelServerEntry;
use crate::database::queries::api_proxy_server_models;

#[derive(Debug)]
pub struct ModelRegistry {
    enabled_models: HashMap<Uuid, ModelServerEntry>,
    alias_map: HashMap<String, Uuid>,
    default_model: Option<Uuid>,
}

impl ModelRegistry {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut registry = Self {
            enabled_models: HashMap::new(),
            alias_map: HashMap::new(),
            default_model: None,
        };

        registry.reload_enabled_models().await?;
        Ok(registry)
    }

    pub async fn reload_enabled_models(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Load enabled models from api_proxy_server_models table
        let enabled_models = api_proxy_server_models::get_enabled_proxy_models().await?;

        self.enabled_models.clear();
        self.alias_map.clear();
        self.default_model = None;

        for proxy_model in enabled_models {
            let model_entry = ModelServerEntry {
                model_id: proxy_model.model_id,
                alias_id: proxy_model.alias_id.clone(),
                enabled: proxy_model.enabled,
            };

            self.enabled_models
                .insert(proxy_model.model_id, model_entry);

            // Build alias map
            if let Some(alias) = &proxy_model.alias_id {
                self.alias_map.insert(alias.clone(), proxy_model.model_id);
            }

            // Set default model
            if proxy_model.is_default {
                self.default_model = Some(proxy_model.model_id);
            }
        }

        tracing::info!(
            "Loaded {} enabled models for API proxy",
            self.enabled_models.len()
        );
        Ok(())
    }

    pub async fn resolve_model_identifier(&self, identifier: &str) -> Result<Uuid, ProxyError> {
        // Try to parse as UUID first
        if let Ok(uuid) = Uuid::parse_str(identifier) {
            return Ok(uuid);
        }

        // Try to resolve as alias
        if let Some(&model_id) = self.alias_map.get(identifier) {
            return Ok(model_id);
        }

        Err(ProxyError::ModelNotFound(format!(
            "Model not found: {}",
            identifier
        )))
    }

    pub fn is_model_enabled(&self, model_id: &Uuid) -> bool {
        self.enabled_models.contains_key(model_id)
    }

    pub async fn get_default_model(&self) -> Option<Uuid> {
        self.default_model
    }

    pub fn list_enabled_models(&self) -> Vec<ModelServerEntry> {
        self.enabled_models.values().cloned().collect()
    }

    pub async fn get_model_display_name(&self, model_id: &Uuid) -> String {
        // Return alias if available, otherwise return UUID
        if let Some(entry) = self.enabled_models.get(model_id) {
            if let Some(alias) = &entry.alias_id {
                return alias.clone();
            }
        }
        model_id.to_string()
    }

    pub fn get_active_models_count(&self) -> usize {
        self.enabled_models.len()
    }

}
