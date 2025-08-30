use crate::{api, utils, RAG_FILE_STORAGE};

/// Initialize file storage systems
pub async fn initialize_file_storage() -> Result<(), String> {
    // Clear temp directory on startup
    if let Err(e) = utils::model_storage::ModelStorage::clear_temp_directory().await {
        eprintln!("Failed to clear temp directory on startup: {}", e);
    }

    // Initialize file storage
    if let Err(e) = api::files::initialize_file_storage().await {
        eprintln!("Failed to initialize file storage: {:?}", e);
    } else {
        println!("File storage initialized successfully");
    }

    // Initialize RAG file storage
    if let Err(e) = RAG_FILE_STORAGE.initialize().await {
        eprintln!("Failed to initialize RAG file storage: {:?}", e);
    } else {
        println!("RAG file storage initialized successfully");
    }

    Ok(())
}

/// Cleanup file storage resources
pub async fn cleanup_file_storage() {
    // Clear temp directory on shutdown
    if let Err(e) = utils::model_storage::ModelStorage::clear_temp_directory().await {
        eprintln!("Failed to clear temp directory on shutdown: {}", e);
    }
}
