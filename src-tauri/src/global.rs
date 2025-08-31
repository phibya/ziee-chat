// Global variables and configuration for the application

use crate::ai::rag::rag_file_storage::RagFileStorage;
use crate::utils::file_storage::FileStorage;
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

// Application name from environment or default
pub static APP_NAME: Lazy<String> =
    Lazy::new(|| std::env::var("APP_NAME").unwrap_or_else(|_| "ziee".to_string()));

// Application data directory with thread-safe access
pub static APP_DATA_DIR: Lazy<Mutex<PathBuf>> = Lazy::new(|| {
    let default_path = std::env::var("APP_DATA_DIR")
        .unwrap_or_else(|_| {
            // {homedir}/.ziee
            let home_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            home_dir
                .join(".ziee")
                .to_str()
                .unwrap_or_default()
                .to_string()
        })
        .parse()
        .unwrap();
    Mutex::new(default_path)
});

/// Set the application data directory
/// Used during app initialization to override the default path
pub fn set_app_data_dir(path: PathBuf) {
    if let Ok(mut app_data_dir) = APP_DATA_DIR.lock() {
        *app_data_dir = path;
    }
}

/// Get the current application data directory
/// Returns a cloned PathBuf to avoid holding the mutex lock
pub fn get_app_data_dir() -> PathBuf {
    APP_DATA_DIR.lock().unwrap().clone()
}

// Global FILE_STORAGE instance for general file operations
pub static FILE_STORAGE: Lazy<Arc<FileStorage>> =
    Lazy::new(|| Arc::new(FileStorage::new(&get_app_data_dir())));

// Global RAG_FILE_STORAGE instance for RAG-specific file operations
pub static RAG_FILE_STORAGE: Lazy<Arc<RagFileStorage>> =
    Lazy::new(|| Arc::new(RagFileStorage::new(&get_app_data_dir())));

// HTTP port for the API server
pub static HTTP_PORT: Lazy<u16> = Lazy::new(|| get_available_port());

/// Get the HTTP port for the API server
/// This is used by Tauri commands and other parts of the app
pub fn get_http_port() -> u16 {
    *HTTP_PORT
}

/// Check if the application is running in desktop mode (with GUI)
/// Returns false if HEADLESS environment variable is set to "true"
pub fn is_desktop_app() -> bool {
    std::env::var("HEADLESS").unwrap_or_default() != "true"
}

/// Find an available port for the API server
/// Tries PORT environment variable first, then 1430, then finds a random available port
pub fn get_available_port() -> u16 {
    // Try PORT environment variable first
    if let Ok(port_str) = std::env::var("PORT") {
        if let Ok(port) = port_str.parse::<u16>() {
            return port;
        }
    }

    // Try default port 1430
    if std::net::TcpListener::bind("127.0.0.1:1430").is_ok() {
        return 1430;
    }

    // Use portpicker to find a random available port
    portpicker::pick_unused_port().unwrap_or(3000)
}
