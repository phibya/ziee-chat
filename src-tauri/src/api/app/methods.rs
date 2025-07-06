#[tauri::command]
pub fn get_http_port() -> u16 {
    // Get the port from the environment variable or default to 3030
    std::env::var("PORT")
        .unwrap_or_else(|_| "1430".to_string())
        .parse()
        .unwrap_or(1430)
}
