use crate::database;

/// Initialize database and perform startup cleanup
pub async fn initialize_database() -> Result<(), String> {
    // Initialize database
    if let Err(e) = database::initialize_database().await {
        return Err(format!("Failed to initialize database: {}", e));
    }

    // Clean up all download instances on startup
    match database::queries::download_instances::delete_all_downloads().await {
        Ok(count) => {
            if count > 0 {
                println!(
                    "Cleaned up {} download instances from previous session",
                    count
                );
            }
        }
        Err(e) => {
            eprintln!("Failed to clean up download instances: {}", e);
        }
    }

    Ok(())
}

/// Cleanup database resources
pub async fn cleanup_database() {
    database::cleanup_database().await;
}
