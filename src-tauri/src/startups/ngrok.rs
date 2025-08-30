use crate::api;

/// Initialize ngrok tunnel
pub async fn initialize_ngrok() -> Result<(), String> {
    // Try to autostart ngrok tunnel if configured
    if let Err(e) = api::configuration::try_autostart_ngrok_tunnel().await {
        eprintln!("Failed to autostart ngrok tunnel: {}", e);
        // Don't fail startup, just log the error
    }

    Ok(())
}
