use crate::ai;

/// Initialize AI models and related services
pub async fn initialize_ai_models() -> Result<(), String> {
    // Start auto-unload task for local model management
    let auto_unload_config = ai::AutoUnloadConfig::default();
    ai::start_auto_unload_task(auto_unload_config);
    println!("Auto-unload task started for local models");

    // Reconcile model states on startup - check database vs actual processes
    if let Err(e) = ai::reconcile_model_states().await {
        eprintln!("Failed to reconcile model states: {}", e);
        // Don't fail startup, just log the error
    }

    Ok(())
}

/// Cleanup AI model resources
pub async fn cleanup_ai_models() {
    // Stop all running models first
    if let Err(e) = ai::shutdown_all_models().await {
        eprintln!("Failed to shutdown models: {}", e);
        // Continue with other cleanup even if model shutdown fails
    }
}
