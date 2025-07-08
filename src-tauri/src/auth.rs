use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::database::models::*;
use crate::database::queries::users;

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
            jwt_secret: generate_jwt_secret(),
            jwt_expiration_hours: 24 * 7, // 1 week
        }
    }
}

fn generate_jwt_secret() -> String {
    let mut rng = rand::thread_rng();
    let secret: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(secret)
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

    /// Hash password using bcrypt
    pub fn hash_password(&self, password: &str) -> Result<String, bcrypt::BcryptError> {
        hash(password, DEFAULT_COST)
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
        verify(password, hash)
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
        let mut rng = rand::thread_rng();
        let token: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
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

        // Verify password
        if !self
            .verify_password(password, &password_service.bcrypt)
            .map_err(|e| e.to_string())?
        {
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
        // Hash password
        let password_hash = self
            .hash_password(&request.password)
            .map_err(|e| e.to_string())?;

        // Create user
        let user = users::create_user(
            request.username,
            request.email,
            Some(password_hash),
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

    /// Create default admin user for desktop app
    pub async fn create_default_admin_user(&self) -> Result<User, String> {
        let admin_request = CreateUserRequest {
            username: "admin".to_string(),
            email: "admin@domain.com".to_string(),
            password: "admin".to_string(),
            profile: Some(serde_json::json!({"isAdmin": true})),
        };

        self.create_user(admin_request).await
    }

    /// Get or create default admin user for desktop app
    pub async fn get_or_create_default_admin_user(&self) -> Result<User, String> {
        // Try to get existing admin user
        if let Some(admin) = users::get_user_by_username("admin")
            .await
            .map_err(|e| e.to_string())?
        {
            return Ok(admin);
        }

        // Create default admin user
        self.create_default_admin_user().await
    }

    /// Auto-login for desktop app - returns JWT token for default admin
    pub async fn auto_login_desktop(&self) -> Result<LoginResponse, String> {
        let admin_user = self.get_or_create_default_admin_user().await?;

        // Generate JWT token
        let token = self
            .generate_token(&admin_user)
            .map_err(|e| e.to_string())?;

        // Generate login token and store it
        let login_token = self.generate_login_token();
        let when_created = Utc::now().timestamp_millis();
        let expires_at = Utc::now() + Duration::hours(self.config.jwt_expiration_hours);

        users::add_login_token(
            admin_user.id,
            login_token.clone(),
            when_created,
            Some(expires_at),
        )
        .await
        .map_err(|e| e.to_string())?;

        Ok(LoginResponse {
            token,
            user: admin_user,
            expires_at,
        })
    }
}
