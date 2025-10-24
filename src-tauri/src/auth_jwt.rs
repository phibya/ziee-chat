use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::*;
use crate::database::queries::users;
use crate::utils::password;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub email: String,
    pub exp: usize, // Expiration time
    pub iat: usize, // Issued at
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: crate::utils::jwt_secret::get_jwt_secret(),
            jwt_expiration_hours: 24 * 7, // 1 week
        }
    }
}

pub struct AuthService {
    config: AuthConfig,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(AuthConfig::default())
    }

    /// Generate JWT token for user
    pub fn generate_token(&self, user: &User) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.config.jwt_expiration_hours);

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.get_primary_email().unwrap_or_default(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let header = Header::new(Algorithm::HS256);
        let key = EncodingKey::from_secret(self.config.jwt_secret.as_ref());

        encode(&header, &claims, &key)
    }

    /// Verify JWT token and extract claims
    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let key = DecodingKey::from_secret(self.config.jwt_secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<Claims>(token, &key, &validation)?;
        Ok(token_data.claims)
    }

    /// Generate a random login token
    pub fn generate_login_token(&self) -> String {
        let mut rng = rand::rng();
        let token: Vec<u8> = (0..32).map(|_| rng.random()).collect();
        hex::encode(token)
    }

    /// Authenticate user with username/email and password
    pub async fn authenticate_user(
        &self,
        username_or_email: &str,
        password: &str,
    ) -> Result<Option<LoginResponse>, String> {
        // Get user by username or email
        let user = users::get_user_by_username_or_email(username_or_email)
            .await
            .map_err(|e| e.to_string())?;

        let Some(user) = user else {
            return Ok(None);
        };

        // Check if user has password service
        let Some(password_service) = &user.services.password else {
            return Ok(None);
        };

        // Verify password with salt
        if !password::verify_password(password, password_service).map_err(|e| e.to_string())? {
            return Ok(None);
        }

        // Generate JWT token
        let token = self.generate_token(&user).map_err(|e| e.to_string())?;

        // Generate login token and store it
        let login_token = self.generate_login_token();
        let when_created = Utc::now().timestamp_millis();
        let expires_at = Utc::now() + Duration::hours(self.config.jwt_expiration_hours);

        users::add_login_token(user.id, login_token.clone(), when_created, Some(expires_at))
            .await
            .map_err(|e| e.to_string())?;

        Ok(Some(LoginResponse {
            token,
            user,
            expires_at,
        }))
    }

    /// Create a new user with email and password
    pub async fn create_user(&self, request: CreateUserRequest) -> Result<User, String> {
        // Check if username already exists
        if let Ok(Some(_)) = users::get_user_by_username(&request.username).await {
            return Err(format!("Username '{}' is already taken", request.username));
        }

        // Check if email already exists
        if let Ok(Some(_)) = users::get_user_by_email(&request.email).await {
            return Err(format!("Email '{}' is already registered", request.email));
        }

        // Hash password with salt
        let password_service =
            password::hash_password(&request.password).map_err(|e| e.to_string())?;

        // Create user
        let user = users::create_user_with_password_service(
            request.username,
            request.email,
            Some(password_service),
            request.profile,
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(user)
    }

    /// Get user by JWT token
    pub async fn get_user_by_token(&self, token: &str) -> Result<Option<User>, String> {
        let claims = self.verify_token(token).map_err(|e| e.to_string())?;
        let user_id = Uuid::parse_str(&claims.sub).map_err(|e| e.to_string())?;

        let user = users::get_user_by_id(user_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok(user)
    }

    /// Logout user by removing login token
    pub async fn logout_user(&self, token: &str) -> Result<(), String> {
        users::remove_login_token(token)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Create default root user for desktop app
    pub async fn create_default_admin_user(&self) -> Result<User, String> {
        let admin_request = CreateUserRequest {
            username: "root".to_string(),
            email: "root@domain.com".to_string(),
            password: "root".to_string(),
            profile: Some(serde_json::json!({})),
        };

        let user = self.create_user(admin_request).await?;

        // Assign admin user to admin group
        if let Err(e) =
            crate::database::queries::user_groups::assign_user_to_admin_group(user.id).await
        {
            eprintln!("Warning: Failed to assign admin user to admin group: {}", e);
        }

        Ok(user)
    }

    /// Get or create default root user for desktop app
    pub async fn get_default_admin_user(&self) -> Result<User, String> {
        // Try to get existing root user
        if let Some(admin) = users::get_user_by_username("root")
            .await
            .map_err(|e| e.to_string())?
        {
            return Ok(admin);
        }

        // Create default root user
        let user = self.create_default_admin_user().await?;

        // Mark app as initialized for desktop apps
        if let Err(e) = crate::database::queries::configuration::mark_app_initialized().await {
            eprintln!("Warning: Failed to mark app as initialized: {}", e);
        }

        Ok(user)
    }

    /// Verify user password
    pub async fn verify_user_password(&self, user: &User, password: &str) -> Result<bool, String> {
        // Check if user has password service
        if let Some(password_service) = &user.services.password {
            password::verify_password(password, password_service).map_err(|e| e.to_string())
        } else {
            Ok(false)
        }
    }

    /// Update user password
    pub async fn update_user_password(
        &self,
        user_id: &Uuid,
        new_password: &str,
    ) -> Result<(), String> {
        let password_service = password::hash_password(new_password).map_err(|e| e.to_string())?;
        users::reset_user_password_with_service(*user_id, password_service)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}
