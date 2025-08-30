use crate::auth::AuthService;

#[tauri::command]
pub fn get_http_port() -> u16 {
    crate::global::get_http_port()
}

#[tauri::command]
pub async fn get_desktop_auth_token() -> Result<String, String> {
    let auth_service = AuthService::default();

    // Get or create default admin user for desktop
    match auth_service.get_default_admin_user().await {
        Ok(user) => {
            // Generate a valid JWT token for this user
            match auth_service.generate_token(&user) {
                Ok(token) => Ok(token),
                Err(e) => Err(format!("Failed to generate token: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to get admin user: {}", e)),
    }
}

pub fn is_desktop_app() -> bool {
    std::env::var("HEADLESS").unwrap_or_default() != "true"
}
