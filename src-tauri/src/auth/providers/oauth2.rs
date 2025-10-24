// OAuth2/OIDC authentication provider implementation

use async_trait::async_trait;
use oauth2::{
    basic::BasicClient, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};
use openidconnect::{
    core::{CoreClient, CoreIdToken, CoreIdTokenVerifier, CoreProviderMetadata, CoreAuthenticationFlow},
    reqwest::async_http_client,
    AccessTokenHash, IssuerUrl, Nonce,
};
// Import TokenResponse trait separately to avoid conflict with oauth2::TokenResponse
use openidconnect::TokenResponse as _;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AuthError, AuthProviderTrait, AuthResult, OAuthResult, UserAttributes};
use crate::database::models::AuthProvider;
use crate::database::queries::auth_sessions;

/// OAuth2/OIDC provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// OAuth 2.0 client ID
    pub client_id: String,
    /// OAuth 2.0 client secret
    pub client_secret: String,
    /// Authorization endpoint URL
    pub authorization_url: String,
    /// Token endpoint URL
    pub token_url: String,
    /// OIDC issuer URL (for OIDC providers)
    pub issuer_url: Option<String>,
    /// User info endpoint URL (for OAuth 2.0 providers without OIDC)
    pub userinfo_url: Option<String>,
    /// Scopes to request
    pub scopes: Vec<String>,
    /// Attribute mapping for user info
    pub attribute_mapping: OAuth2AttributeMapping,
    /// Session timeout in seconds (default: 300 = 5 minutes)
    pub session_timeout_seconds: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2AttributeMapping {
    pub user_id: String,      // Default: "sub"
    pub username: String,     // Default: "preferred_username" or "email"
    pub email: String,        // Default: "email"
    pub display_name: Option<String>,  // Default: "name"
    pub first_name: Option<String>,    // Default: "given_name"
    pub last_name: Option<String>,     // Default: "family_name"
    pub groups: Option<String>,        // Default: "groups"
}

impl Default for OAuth2AttributeMapping {
    fn default() -> Self {
        Self {
            user_id: "sub".to_string(),
            username: "preferred_username".to_string(),
            email: "email".to_string(),
            display_name: Some("name".to_string()),
            first_name: Some("given_name".to_string()),
            last_name: Some("family_name".to_string()),
            groups: Some("groups".to_string()),
        }
    }
}

pub struct OAuth2Provider {
    name: String,
    provider_id: Uuid,
    config: OAuth2Config,
    raw_config: serde_json::Value,
}

impl OAuth2Provider {
    pub fn new(provider: &AuthProvider) -> Result<Self, AuthError> {
        let config: OAuth2Config = serde_json::from_value(provider.config.clone())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid OAuth2 configuration: {}", e)))?;

        Ok(Self {
            name: provider.name.clone(),
            provider_id: provider.id,
            config,
            raw_config: provider.config.clone(),
        })
    }

    fn create_client(&self, redirect_uri: &str) -> Result<BasicClient, AuthError> {
        let client_id = ClientId::new(self.config.client_id.clone());
        let client_secret = ClientSecret::new(self.config.client_secret.clone());
        let auth_url = AuthUrl::new(self.config.authorization_url.clone())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid authorization URL: {}", e)))?;
        let token_url = TokenUrl::new(self.config.token_url.clone())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid token URL: {}", e)))?;
        let redirect_url = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid redirect URL: {}", e)))?;

        Ok(BasicClient::new(
            client_id,
            Some(client_secret),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url))
    }

    async fn create_oidc_client(&self, redirect_uri: &str) -> Result<CoreClient, AuthError> {
        let issuer_url = self.config.issuer_url.as_ref()
            .ok_or_else(|| AuthError::ConfigurationError("OIDC issuer URL not configured".to_string()))?;

        let issuer = IssuerUrl::new(issuer_url.clone())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid issuer URL: {}", e)))?;

        let metadata = CoreProviderMetadata::discover_async(issuer, async_http_client)
            .await
            .map_err(|e| AuthError::ConfigurationError(format!("Failed to discover OIDC metadata: {}", e)))?;

        let client_id = ClientId::new(self.config.client_id.clone());
        let client_secret = ClientSecret::new(self.config.client_secret.clone());
        let redirect_url = RedirectUrl::new(redirect_uri.to_string())
            .map_err(|e| AuthError::ConfigurationError(format!("Invalid redirect URL: {}", e)))?;

        Ok(CoreClient::from_provider_metadata(
            metadata,
            client_id,
            Some(client_secret),
        )
        .set_redirect_uri(redirect_url))
    }

    async fn get_user_info_from_token(&self, id_token: &CoreIdToken, verifier: &CoreIdTokenVerifier<'_>, nonce: &Nonce) -> Result<serde_json::Value, AuthError> {
        let claims = id_token
            .claims(verifier, nonce)
            .map_err(|e| AuthError::InvalidCredentials(format!("ID token verification failed: {}", e)))?;

        Ok(serde_json::json!({
            "sub": claims.subject().to_string(),
            "email": claims.email().map(|e| e.as_str()),
            "email_verified": claims.email_verified(),
            "name": claims.name().and_then(|n| n.get(None)).map(|n| n.as_str()),
            "given_name": claims.given_name().and_then(|n| n.get(None)).map(|n| n.as_str()),
            "family_name": claims.family_name().and_then(|n| n.get(None)).map(|n| n.as_str()),
            "preferred_username": claims.preferred_username().map(|u| u.as_str()),
        }))
    }

    async fn get_user_info_from_api(&self, access_token: &str) -> Result<serde_json::Value, AuthError> {
        let userinfo_url = self.config.userinfo_url.as_ref()
            .ok_or_else(|| AuthError::ConfigurationError("UserInfo URL not configured".to_string()))?;

        let client = reqwest::Client::new();
        let response = client
            .get(userinfo_url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::ConnectionFailed(format!("Failed to fetch user info: {}", e)))?;

        if !response.status().is_success() {
            return Err(AuthError::InternalError(format!(
                "UserInfo request failed with status: {}",
                response.status()
            )));
        }

        response
            .json::<serde_json::Value>()
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to parse user info: {}", e)))
    }

    fn extract_user_attributes(&self, user_info: &serde_json::Value) -> Result<UserAttributes, AuthError> {
        let get_str = |key: &str| -> Option<String> {
            user_info.get(key)
                .and_then(|v| v.as_str())
                .map(String::from)
        };

        let get_str_array = |key: &str| -> Vec<String> {
            user_info.get(key)
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect())
                .unwrap_or_default()
        };

        let username = get_str(&self.config.attribute_mapping.username)
            .or_else(|| get_str(&self.config.attribute_mapping.email))
            .ok_or_else(|| AuthError::InternalError("No username found in user info".to_string()))?;

        let email = get_str(&self.config.attribute_mapping.email)
            .unwrap_or_default();

        let display_name = self.config.attribute_mapping.display_name.as_ref()
            .and_then(|attr| get_str(attr));
        let first_name = self.config.attribute_mapping.first_name.as_ref()
            .and_then(|attr| get_str(attr));
        let last_name = self.config.attribute_mapping.last_name.as_ref()
            .and_then(|attr| get_str(attr));

        let groups = self.config.attribute_mapping.groups.as_ref()
            .map(|attr| get_str_array(attr))
            .unwrap_or_default();

        Ok(UserAttributes {
            username,
            email,
            display_name,
            first_name,
            last_name,
            groups,
        })
    }
}

#[async_trait]
impl AuthProviderTrait for OAuth2Provider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> &str {
        if self.config.issuer_url.is_some() {
            "oidc"
        } else {
            "oauth2"
        }
    }

    async fn authenticate(
        &self,
        _username: &str,
        _password: &str,
    ) -> Result<AuthResult, AuthError> {
        Err(AuthError::NotSupported(
            "OAuth2/OIDC does not support password authentication".to_string(),
        ))
    }

    async fn init_oauth_flow(&self, redirect_uri: &str) -> Result<OAuthResult, AuthError> {
        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Generate state and nonce
        let state = CsrfToken::new_random();
        let nonce = Nonce::new_random();

        // Build authorization URL
        let auth_url = if self.config.issuer_url.is_some() {
            // OIDC flow
            let client = self.create_oidc_client(redirect_uri).await?;

            // Clone values for the closures
            let state_val = state.clone();
            let nonce_val = nonce.clone();

            let mut auth_request = client
                .authorize_url(
                    CoreAuthenticationFlow::AuthorizationCode,
                    move || state_val.clone(),
                    move || nonce_val.clone(),
                )
                .set_pkce_challenge(pkce_challenge);

            // Add scopes
            for scope in &self.config.scopes {
                auth_request = auth_request.add_scope(Scope::new(scope.clone()));
            }

            let (url, _, _) = auth_request.url();
            url
        } else {
            // OAuth 2.0 flow
            let client = self.create_client(redirect_uri)?;
            let mut auth_request = client
                .authorize_url(|| state.clone())
                .set_pkce_challenge(pkce_challenge);

            // Add scopes
            for scope in &self.config.scopes {
                auth_request = auth_request.add_scope(Scope::new(scope.clone()));
            }

            let (url, _) = auth_request.url();
            url
        };

        // Create session
        let session_key = Uuid::new_v4().to_string();
        let timeout_seconds = self.config.session_timeout_seconds.unwrap_or(300);

        auth_sessions::create_session(
            &session_key,
            self.provider_id,
            state.secret(),
            Some(nonce.secret()),
            Some(pkce_verifier.secret()),
            Some(redirect_uri),
            serde_json::json!({}),
            timeout_seconds,
        )
        .await
        .map_err(|e| AuthError::InternalError(format!("Failed to create session: {}", e)))?;

        Ok(OAuthResult {
            redirect_url: auth_url.to_string(),
            session_key,
        })
    }

    async fn handle_oauth_callback(
        &self,
        code: &str,
        state: &str,
        session_key: &str,
    ) -> Result<AuthResult, AuthError> {
        // Get and validate session
        let session = auth_sessions::get_by_session_key(session_key)
            .await
            .map_err(|e| AuthError::InternalError(format!("Failed to get session: {}", e)))?
            .ok_or_else(|| AuthError::InvalidCredentials("Invalid or expired session".to_string()))?;

        if session.state != state {
            return Err(AuthError::InvalidCredentials("State mismatch".to_string()));
        }

        if session.provider_id != self.provider_id {
            return Err(AuthError::InvalidCredentials("Provider mismatch".to_string()));
        }

        let redirect_uri = session.redirect_uri.as_ref()
            .ok_or_else(|| AuthError::InternalError("No redirect URI in session".to_string()))?;

        let pkce_verifier = session.code_verifier.as_ref()
            .ok_or_else(|| AuthError::InternalError("No PKCE verifier in session".to_string()))?;

        // Exchange code for token
        let (_access_token, user_info) = if self.config.issuer_url.is_some() {
            // OIDC flow
            let client = self.create_oidc_client(redirect_uri).await?;
            let nonce_str = session.nonce.as_ref()
                .ok_or_else(|| AuthError::InternalError("No nonce in session".to_string()))?;

            let token_response = client
                .exchange_code(AuthorizationCode::new(code.to_string()))
                .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.clone()))
                .request_async(async_http_client)
                .await
                .map_err(|e| AuthError::InvalidCredentials(format!("Token exchange failed: {}", e)))?;

            let id_token = token_response
                .id_token()
                .ok_or_else(|| AuthError::InternalError("No ID token in response".to_string()))?;

            // Verify ID token
            let verifier = client.id_token_verifier();
            let nonce = Nonce::new(nonce_str.clone());
            let claims = id_token
                .claims(&verifier, &nonce)
                .map_err(|e| AuthError::InvalidCredentials(format!("ID token verification failed: {}", e)))?;

            // Verify access token hash if present
            if let Some(at_hash) = claims.access_token_hash() {
                let actual_hash = AccessTokenHash::from_token(
                    token_response.access_token(),
                    &id_token.signing_alg()
                        .map_err(|e| AuthError::InternalError(format!("Failed to get signing algorithm: {}", e)))?,
                )
                .map_err(|e| AuthError::InternalError(format!("Failed to compute access token hash: {}", e)))?;

                if actual_hash != *at_hash {
                    return Err(AuthError::InvalidCredentials("Access token hash mismatch".to_string()));
                }
            }

            let user_info = self.get_user_info_from_token(id_token, &verifier, &nonce).await?;
            (token_response.access_token().secret().clone(), user_info)
        } else {
            // OAuth 2.0 flow
            let client = self.create_client(redirect_uri)?;
            let token_response = client
                .exchange_code(AuthorizationCode::new(code.to_string()))
                .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.clone()))
                .request_async(async_http_client)
                .await
                .map_err(|e| AuthError::InvalidCredentials(format!("Token exchange failed: {}", e)))?;

            let access_token = token_response.access_token().secret().clone();
            let user_info = self.get_user_info_from_api(&access_token).await?;
            (access_token, user_info)
        };

        // Extract user attributes
        let attributes = self.extract_user_attributes(&user_info)?;

        let external_id = user_info
            .get(&self.config.attribute_mapping.user_id)
            .and_then(|v| v.as_str())
            .ok_or_else(|| AuthError::InternalError("No user ID in token".to_string()))?
            .to_string();

        // Delete session after successful authentication
        let _ = auth_sessions::delete_session(session_key).await;

        Ok(AuthResult {
            external_id,
            external_username: Some(attributes.username.clone()),
            external_email: Some(attributes.email.clone()),
            metadata: serde_json::json!({
                "provider": self.provider_type(),
                "auth_method": "oauth2",
                "user_info": user_info,
            }),
            attributes,
        })
    }

    async fn test_connection(&self) -> Result<(), AuthError> {
        // Test by attempting to discover OIDC metadata if OIDC provider
        if let Some(issuer_url) = &self.config.issuer_url {
            let issuer = IssuerUrl::new(issuer_url.clone())
                .map_err(|e| AuthError::ConfigurationError(format!("Invalid issuer URL: {}", e)))?;

            CoreProviderMetadata::discover_async(issuer, async_http_client)
                .await
                .map_err(|e| AuthError::ConnectionFailed(format!("Failed to discover OIDC metadata: {}", e)))?;
        } else {
            // For OAuth 2.0, just validate URLs
            AuthUrl::new(self.config.authorization_url.clone())
                .map_err(|e| AuthError::ConfigurationError(format!("Invalid authorization URL: {}", e)))?;
            TokenUrl::new(self.config.token_url.clone())
                .map_err(|e| AuthError::ConfigurationError(format!("Invalid token URL: {}", e)))?;
        }

        Ok(())
    }

    fn get_config(&self) -> &serde_json::Value {
        &self.raw_config
    }
}
