use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{AuthError, AuthProviderTrait, AuthResult};
use crate::database::models::AuthProvider;

/// SAML 2.0 provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlConfig {
    /// SAML Identity Provider (IdP) entity ID
    pub idp_entity_id: String,
    /// SAML IdP SSO URL
    pub idp_sso_url: String,
    /// SAML IdP certificate (PEM format)
    pub idp_certificate: String,
    /// Service Provider (SP) entity ID
    pub sp_entity_id: String,
    /// Assertion Consumer Service (ACS) URL
    pub acs_url: String,
    /// SP private key (PEM format)
    pub sp_private_key: Option<String>,
    /// SP certificate (PEM format)
    pub sp_certificate: Option<String>,
    /// Attribute mapping
    pub attribute_mapping: SamlAttributeMapping,
    /// Whether to sign authentication requests
    pub sign_requests: bool,
    /// Whether to require signed assertions
    pub require_signed_assertions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlAttributeMapping {
    pub user_id: String,         // Default: "urn:oid:0.9.2342.19200300.100.1.1" (uid)
    pub username: String,        // Default: "urn:oid:0.9.2342.19200300.100.1.1"
    pub email: String,           // Default: "urn:oid:0.9.2342.19200300.100.1.3" (mail)
    pub display_name: Option<String>,  // Default: "urn:oid:2.16.840.1.113730.3.1.241" (displayName)
    pub first_name: Option<String>,    // Default: "urn:oid:2.5.4.42" (givenName)
    pub last_name: Option<String>,     // Default: "urn:oid:2.5.4.4" (sn)
    pub groups: Option<String>,        // Default: "urn:oid:1.3.6.1.4.1.5923.1.5.1.1" (eduPersonAffiliation)
}

impl Default for SamlAttributeMapping {
    fn default() -> Self {
        Self {
            user_id: "urn:oid:0.9.2342.19200300.100.1.1".to_string(),
            username: "urn:oid:0.9.2342.19200300.100.1.1".to_string(),
            email: "urn:oid:0.9.2342.19200300.100.1.3".to_string(),
            display_name: Some("urn:oid:2.16.840.1.113730.3.1.241".to_string()),
            first_name: Some("urn:oid:2.5.4.42".to_string()),
            last_name: Some("urn:oid:2.5.4.4".to_string()),
            groups: Some("urn:oid:1.3.6.1.4.1.5923.1.5.1.1".to_string()),
        }
    }
}

pub struct SamlProvider {
    name: String,
    config: SamlConfig,
    raw_config: serde_json::Value,
}

impl SamlProvider {
    pub fn new(provider: &AuthProvider) -> Result<Self, AuthError> {
        let config: SamlConfig = serde_json::from_value(provider.config.clone())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid SAML configuration: {}", e)))?;

        Ok(Self {
            name: provider.name.clone(),
            config,
            raw_config: provider.config.clone(),
        })
    }
}

#[async_trait]
impl AuthProviderTrait for SamlProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> &str {
        "saml"
    }

    async fn authenticate(
        &self,
        _username: &str,
        _password: &str,
    ) -> Result<AuthResult, AuthError> {
        Err(AuthError::NotSupported(
            "SAML does not support password authentication".to_string(),
        ))
    }

    async fn test_connection(&self) -> Result<(), AuthError> {
        // For SAML, validate configuration
        if self.config.idp_entity_id.is_empty() {
            return Err(AuthError::ConfigurationError("IdP entity ID is required".to_string()));
        }
        if self.config.idp_sso_url.is_empty() {
            return Err(AuthError::ConfigurationError("IdP SSO URL is required".to_string()));
        }
        if self.config.idp_certificate.is_empty() {
            return Err(AuthError::ConfigurationError("IdP certificate is required".to_string()));
        }
        if self.config.sp_entity_id.is_empty() {
            return Err(AuthError::ConfigurationError("SP entity ID is required".to_string()));
        }
        if self.config.acs_url.is_empty() {
            return Err(AuthError::ConfigurationError("ACS URL is required".to_string()));
        }

        // Note: Full SAML implementation would validate certificates and potentially
        // fetch IdP metadata, but this is a stub for now
        Ok(())
    }

    fn get_config(&self) -> &serde_json::Value {
        &self.raw_config
    }
}
