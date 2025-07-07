use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Database table structures (for direct DB operations)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserDb {
    pub id: Uuid,
    pub username: String,
    pub created_at: DateTime<Utc>,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserEmailDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub address: String,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserServiceDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub service_name: String,
    pub service_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserLoginTokenDb {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub when_created: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Meteor-like User structure (for API responses)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub emails: Vec<UserEmail>,
    pub created_at: DateTime<Utc>,
    pub profile: Option<serde_json::Value>,
    pub services: UserServices,
}

// Email structure for the emails array
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmail {
    pub address: String,
    pub verified: bool,
}

// Login token structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginToken {
    pub token: String,
    pub when: i64, // Unix timestamp in milliseconds
}

// Service structures for the services object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacebookService {
    pub id: String,
    pub access_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeService {
    pub login_tokens: Vec<LoginToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordService {
    pub bcrypt: String, // bcrypt hash of the password
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserServices {
    pub facebook: Option<FacebookService>,
    pub resume: Option<ResumeService>,
    pub password: Option<PasswordService>,
}

// Helper structures for API requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
    pub expires_at: DateTime<Utc>,
}

// Helper functions for working with the Meteor-like structure
impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            username,
            emails: vec![UserEmail {
                address: email,
                verified: false,
            }],
            created_at: Utc::now(),
            profile: None,
            services: UserServices::default(),
        }
    }
    
    pub fn get_primary_email(&self) -> Option<String> {
        self.emails.first().map(|e| e.address.clone())
    }
    
    pub fn add_email(&mut self, email: String, verified: bool) {
        self.emails.push(UserEmail {
            address: email,
            verified,
        });
    }
    
    pub fn verify_email(&mut self, email: &str) -> bool {
        for email_obj in &mut self.emails {
            if email_obj.address == email {
                email_obj.verified = true;
                return true;
            }
        }
        false
    }
    
    pub fn set_password(&mut self, password_hash: String) {
        self.services.password = Some(PasswordService {
            bcrypt: password_hash,
        });
    }
    
    pub fn add_login_token(&mut self, token: String) {
        let login_token = LoginToken {
            token,
            when: Utc::now().timestamp_millis(),
        };
        
        if let Some(resume) = &mut self.services.resume {
            resume.login_tokens.push(login_token);
        } else {
            self.services.resume = Some(ResumeService {
                login_tokens: vec![login_token],
            });
        }
    }
    
    // Convert from database structures to Meteor-like User
    pub fn from_db_parts(
        user_db: UserDb,
        emails: Vec<UserEmailDb>,
        services: Vec<UserServiceDb>,
        login_tokens: Vec<UserLoginTokenDb>,
    ) -> Self {
        let mut user = User {
            id: user_db.id,
            username: user_db.username,
            emails: emails.into_iter().map(|e| UserEmail {
                address: e.address,
                verified: e.verified,
            }).collect(),
            created_at: user_db.created_at,
            profile: user_db.profile,
            services: UserServices::default(),
        };
        
        // Build services from database records
        for service in services {
            match service.service_name.as_str() {
                "facebook" => {
                    if let Ok(fb_service) = serde_json::from_value::<FacebookService>(service.service_data) {
                        user.services.facebook = Some(fb_service);
                    }
                }
                "password" => {
                    if let Ok(pwd_service) = serde_json::from_value::<PasswordService>(service.service_data) {
                        user.services.password = Some(pwd_service);
                    }
                }
                _ => {}
            }
        }
        
        // Add login tokens to resume service
        if !login_tokens.is_empty() {
            let tokens: Vec<LoginToken> = login_tokens.into_iter().map(|t| LoginToken {
                token: t.token,
                when: t.when_created,
            }).collect();
            
            user.services.resume = Some(ResumeService {
                login_tokens: tokens,
            });
        }
        
        user
    }
}