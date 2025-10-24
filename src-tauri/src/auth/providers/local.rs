use async_trait::async_trait;

use super::{AuthError, AuthProviderTrait, AuthResult, UserAttributes};
use crate::database::models::AuthProvider;
use crate::database::queries::users;
use crate::utils::password;

/// Local authentication provider using database-stored passwords
pub struct LocalAuthProvider {
    name: String,
    config: serde_json::Value,
}

impl LocalAuthProvider {
    pub fn new(provider: &AuthProvider) -> Result<Self, AuthError> {
        Ok(Self {
            name: provider.name.clone(),
            config: provider.config.clone(),
        })
    }
}

#[async_trait]
impl AuthProviderTrait for LocalAuthProvider {
    fn name(&self) -> &str {
        &self.name
    }

    fn provider_type(&self) -> &str {
        "local"
    }

    async fn authenticate(
        &self,
        username: &str,
        password: &str,
    ) -> Result<AuthResult, AuthError> {
        // Get user by username or email
        let user = users::get_user_by_username_or_email(username)
            .await
            .map_err(|e| AuthError::InternalError(format!("Database error: {}", e)))?
            .ok_or_else(|| AuthError::InvalidCredentials("User not found".to_string()))?;

        // Check if user has password service
        let password_service = user.services.password.as_ref().ok_or_else(|| {
            AuthError::InvalidCredentials("No password configured for this user".to_string())
        })?;

        // Verify password with salt
        let valid = password::verify_password(password, password_service)
            .map_err(|e| AuthError::InternalError(format!("Password verification error: {}", e)))?;

        if !valid {
            return Err(AuthError::InvalidCredentials("Invalid password".to_string()));
        }

        // Return auth result
        Ok(AuthResult {
            external_id: user.id.to_string(),
            external_username: Some(user.username.clone()),
            external_email: Some(user.get_primary_email().unwrap_or_default()),
            metadata: serde_json::json!({
                "provider": "local",
                "auth_method": "password"
            }),
            attributes: UserAttributes {
                username: user.username.clone(),
                email: user.get_primary_email().unwrap_or_default(),
                display_name: user.profile.as_ref()
                    .and_then(|p| p.get("display_name"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                first_name: user.profile.as_ref()
                    .and_then(|p| p.get("first_name"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                last_name: user.profile.as_ref()
                    .and_then(|p| p.get("last_name"))
                    .and_then(|v| v.as_str())
                    .map(String::from),
                groups: user.groups.iter().map(|g| g.name.clone()).collect(),
            },
        })
    }

    async fn test_connection(&self) -> Result<(), AuthError> {
        // For local provider, just verify database connectivity
        users::get_user_by_username("__test_connection__")
            .await
            .map_err(|e| AuthError::ConnectionFailed(format!("Database connection failed: {}", e)))?;

        Ok(())
    }

    fn get_config(&self) -> &serde_json::Value {
        &self.config
    }
}
