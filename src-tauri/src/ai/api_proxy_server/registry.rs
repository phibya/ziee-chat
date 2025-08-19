use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use super::{HttpForwardingProvider, ProxyError};
use crate::ai::providers::{AnthropicProvider, OpenAICompatibleProvider};
use crate::database::models::api_proxy_server_model::ModelServerEntry;
use crate::database::queries::{api_proxy_server_models, models, providers};
use crate::utils::proxy::create_proxy_config;

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

    /// Get HTTP forwarding provider for a model
    pub async fn get_forwarding_provider(
        &self,
        model_id: &Uuid,
    ) -> Result<Arc<dyn HttpForwardingProvider>, ProxyError> {
        // Check if model is enabled in proxy
        if !self.is_model_enabled(model_id) {
            return Err(ProxyError::ModelNotInProxy(*model_id));
        }

        // Load model and provider from database
        let model = models::get_model_by_id(*model_id)
            .await
            .map_err(|e| ProxyError::DatabaseError(e.to_string()))?
            .ok_or(ProxyError::ModelNotFound(model_id.to_string()))?;

        let provider = providers::get_provider_by_id(model.provider_id)
            .await
            .map_err(|e| ProxyError::DatabaseError(e.to_string()))?
            .ok_or(ProxyError::ProviderNotFound(model.provider_id))?;

        // Create forwarding provider based on type
        let forwarding_provider: Arc<dyn HttpForwardingProvider> =
            match provider.provider_type.as_str() {
                "openai" | "groq" | "deepseek" | "mistral" | "custom" => {
                    let base_url = provider
                        .base_url
                        .ok_or(ProxyError::RemoteProviderMissingBaseUrl(provider.id))?;

                    Arc::new(
                        OpenAICompatibleProvider::new(
                            provider.api_key.unwrap_or_default(),
                            base_url,
                            match provider.provider_type.as_str() {
                                "openai" => "openai",
                                "groq" => "groq",
                                "deepseek" => "deepseek",
                                "mistral" => "mistral",
                                "custom" => "custom",
                                _ => "openai",
                            },
                            provider
                                .proxy_settings
                                .as_ref()
                                .and_then(create_proxy_config),
                            provider.id,
                        )
                        .map_err(|e| ProxyError::DatabaseError(e.to_string()))?,
                    )
                }
                "anthropic" => {
                    let base_url = provider
                        .base_url
                        .ok_or(ProxyError::RemoteProviderMissingBaseUrl(provider.id))?;

                    Arc::new(
                        AnthropicProvider::new(
                            provider.api_key.unwrap_or_default(),
                            Some(base_url),
                            provider
                                .proxy_settings
                                .as_ref()
                                .and_then(create_proxy_config),
                            provider.id,
                        )
                        .map_err(|e| ProxyError::DatabaseError(e.to_string()))?,
                    )
                }
                _ => return Err(ProxyError::ProviderNotFound(provider.id)),
            };

        Ok(forwarding_provider)
    }
}
