use crate::{
    get_app_data_dir,
    utils::hub_manager::{HubManager, HUB_MANAGER},
};

/// Initialize hub manager
pub async fn initialize_hub() -> Result<(), String> {
    // Initialize hub manager
    match HubManager::new(get_app_data_dir()) {
        Ok(hub_manager) => {
            if let Err(e) = hub_manager.initialize().await {
                eprintln!("Failed to initialize hub manager: {}", e);
            } else {
                println!("Hub manager initialized successfully");
                // Store hub manager globally
                let mut global_hub = HUB_MANAGER.lock().await;
                *global_hub = Some(hub_manager);
            }
        }
        Err(e) => {
            eprintln!("Failed to create hub manager: {}", e);
        }
    }

    Ok(())
}
