use crate::utils::resource_paths::ResourcePaths;

/// Initialize environment variables required for the application
pub fn initialize_environment() {
    set_tessdata_prefix();
}

/// Set TESSDATA_PREFIX environment variable based on the platform
fn set_tessdata_prefix() {
    let tessdata_path = ResourcePaths::find_resource_folder("tessdata");
    
    // SAFETY: Setting environment variables is safe in this context as we're only
    // setting a path variable during application initialization, before any threads
    // that might read this variable are spawned.
    unsafe {
        std::env::set_var("TESSDATA_PREFIX", &tessdata_path);
    }
    
    println!("Set TESSDATA_PREFIX to: {}", tessdata_path.display());
}
