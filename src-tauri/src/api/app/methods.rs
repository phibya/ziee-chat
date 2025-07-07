#[tauri::command]
pub fn get_http_port() -> u16 {
    *crate::HTTP_PORT
}

pub fn is_desktop_app() -> bool {
    std::env::var("HEADLESS").unwrap_or_default() != "true"
}
