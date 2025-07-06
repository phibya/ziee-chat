#[tauri::command]
pub fn get_http_port() -> u16 {
    *crate::HTTP_PORT
}
