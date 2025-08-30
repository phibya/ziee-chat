use crate::ai;

/// Initialize API proxy server
pub async fn initialize_api_proxy() -> Result<(), String> {
    // Try to autostart the API proxy server if configured
    if let Err(e) = ai::api_proxy_server::try_autostart_proxy_server().await {
        eprintln!("Failed to autostart API proxy server: {}", e);
        // Don't fail startup, just log the error
    }

    Ok(())
}
