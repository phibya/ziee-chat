use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::providers::{
    create_provider, AuthError, AuthProviderTrait, AuthResult, OAuthResult,
};
use crate::database::models::AuthProvider;
use crate::database::queries::auth_providers;

/// Authentication service that manages all providers and orchestrates authentication
pub struct AuthService {
    providers: Arc<RwLock<HashMap<Uuid, Arc<Box<dyn AuthProviderTrait>>>>>,
}

impl AuthService {
    /// Create a new AuthService instance
    pub fn new() -> Self {
        Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the service by loading all enabled providers
    pub async fn initialize(&self) -> Result<(), AuthError> {
        let providers = auth_providers::list_all()
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to load providers: {}", e)))?;

        let mut provider_map = self.providers.write().await;
        provider_map.clear();

        for config in providers {
            if config.enabled {
                match create_provider(&config) {
                    Ok(provider) => {
                        provider_map.insert(config.id, Arc::new(provider));
                    }
                    Err(e) => {
                        eprintln!("Failed to initialize provider '{}': {}", config.name, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Reload providers from database (call after provider configuration changes)
    pub async fn reload_providers(&self) -> Result<(), AuthError> {
        self.initialize().await
    }

    /// Get a provider by ID
    pub async fn get_provider(&self, provider_id: Uuid) -> Result<Arc<Box<dyn AuthProviderTrait>>, AuthError> {
        let providers = self.providers.read().await;
        providers
            .get(&provider_id)
            .cloned()
            .ok_or_else(|| AuthError::ProviderDisabled(format!("Provider {} not found or disabled", provider_id)))
    }

    /// Get a provider by name
    pub async fn get_provider_by_name(&self, name: &str) -> Result<Arc<Box<dyn AuthProviderTrait>>, AuthError> {
        let provider_config = auth_providers::get_by_name(name)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get provider: {}", e)))?
            .ok_or_else(|| AuthError::ProviderDisabled(format!("Provider '{}' not found", name)))?;

        self.get_provider(provider_config.id).await
    }

    /// List all enabled provider configurations
    pub async fn list_enabled_providers(&self) -> Result<Vec<AuthProvider>, AuthError> {
        auth_providers::list_enabled()
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to list enabled providers: {}", e)))
    }

    /// Discover provider for a given username/email
    /// Returns the best matching provider based on domain mapping rules
    pub async fn discover_provider(&self, identifier: &str) -> Result<Option<Uuid>, AuthError> {
        let providers = auth_providers::list_enabled()
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to list providers: {}", e)))?;

        // Sort by priority (higher priority first)
        let mut sorted_providers = providers;
        sorted_providers.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Check each provider's mapping rules
        for provider in sorted_providers {
            if let Some(domain_mappings) = provider.mapping_rules.get("domain_mappings") {
                if let Some(domains) = domain_mappings.as_array() {
                    for domain in domains {
                        if let Some(domain_str) = domain.as_str() {
                            if identifier.ends_with(&format!("@{}", domain_str)) {
                                return Ok(Some(provider.id));
                            }
                        }
                    }
                }
            }
        }

        // Return local provider as fallback if no match found
        let local_provider = auth_providers::get_by_type("local")
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get local provider: {}", e)))?;

        Ok(local_provider.map(|p| p.id))
    }

    /// Authenticate a user with username/password using a specific provider
    pub async fn authenticate_with_provider(
        &self,
        provider_id: Uuid,
        username: &str,
        password: &str,
    ) -> Result<AuthResult, AuthError> {
        let provider = self.get_provider(provider_id).await?;
        provider.authenticate(username, password).await
    }

    /// Authenticate a user with automatic provider discovery
    pub async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResult, AuthError> {
        // Discover provider for username
        let provider_id = self.discover_provider(username)
            .await?
            .ok_or_else(|| AuthError::UserNotFound("No provider found for user".to_string()))?;

        self.authenticate_with_provider(provider_id, username, password).await
    }

    /// Initialize OAuth flow for a provider
    pub async fn init_oauth_flow(
        &self,
        provider_id: Uuid,
        redirect_uri: &str,
    ) -> Result<OAuthResult, AuthError> {
        let provider = self.get_provider(provider_id).await?;
        provider.init_oauth_flow(redirect_uri).await
    }

    /// Handle OAuth callback
    pub async fn handle_oauth_callback(
        &self,
        provider_id: Uuid,
        code: &str,
        state: &str,
        session_key: &str,
    ) -> Result<AuthResult, AuthError> {
        let provider = self.get_provider(provider_id).await?;
        provider.handle_oauth_callback(code, state, session_key).await
    }

    /// Test connection to a provider
    pub async fn test_provider_connection(&self, provider_id: Uuid) -> Result<(), AuthError> {
        let provider = self.get_provider(provider_id).await?;
        provider.test_connection().await
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new()
    }
}

/// Global AuthService instance
static AUTH_SERVICE: once_cell::sync::Lazy<AuthService> = once_cell::sync::Lazy::new(AuthService::new);

/// Get the global AuthService instance
pub fn get_auth_service() -> &'static AuthService {
    &AUTH_SERVICE
}

/// Initialize the global AuthService
pub async fn initialize_auth_service() -> Result<(), AuthError> {
    AUTH_SERVICE.initialize().await
}

/// Reload providers in the global AuthService
pub async fn reload_auth_providers() -> Result<(), AuthError> {
    AUTH_SERVICE.reload_providers().await
}
