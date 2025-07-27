use once_cell::sync::Lazy;
use std::sync::Mutex;
use rand::Rng;
use hex;

// Global JWT secret that persists for the app's lifetime
static JWT_SECRET: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

/// Get the JWT secret - either from environment or generate a persistent one
pub fn get_jwt_secret() -> String {
    // First try to get from environment
    if let Ok(secret) = std::env::var("JWT_SECRET") {
        return secret;
    }

    // If not in environment, get or create a persistent one for this app session
    let mut secret_guard = JWT_SECRET.lock().unwrap();
    
    if let Some(ref secret) = *secret_guard {
        secret.clone()
    } else {
        // Generate a new secret and store it for the session
        let new_secret = generate_jwt_secret();
        *secret_guard = Some(new_secret.clone());
        new_secret
    }
}

/// Generate a random JWT secret
fn generate_jwt_secret() -> String {
    let mut rng = rand::rng();
    let secret: Vec<u8> = (0..32).map(|_| rng.random()).collect();
    hex::encode(secret)
}