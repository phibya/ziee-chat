use crate::ai::core::providers::FileReference;
use crate::database::queries::files::*;
use crate::utils::file_storage::extract_extension;
use crate::FILE_STORAGE;
use std::collections::HashMap;
use uuid::Uuid;

/// Load FileReference from database
pub async fn load_file_reference(file_id: Uuid) -> Result<FileReference, Box<dyn std::error::Error + Send + Sync>> {
    // Get file info using existing database function
    let file = get_file_by_id(file_id)
        .await?
        .ok_or("File not found")?;
    
    Ok(FileReference {
        file_id: file.id,
        filename: file.filename,
        file_size: file.file_size,
        mime_type: file.mime_type,
        checksum: file.checksum,
    })
}

/// Get provider file mappings for a file
pub async fn get_provider_file_mappings(file_id: Uuid) -> Result<HashMap<Uuid, String>, Box<dyn std::error::Error + Send + Sync>> {
    // Use existing database function to get provider files
    let provider_files = get_provider_file_mappings_by_file(file_id).await?;
    
    let mut mappings = HashMap::new();
    for provider_file in provider_files {
        if let Some(provider_file_id) = provider_file.provider_file_id {
            mappings.insert(provider_file.provider_id, provider_file_id);
        }
    }
    
    Ok(mappings)
}

/// Create or update provider file mapping
pub async fn save_provider_file_mapping(file_id: Uuid, provider_id: Uuid, provider_file_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use existing database function to create provider file mapping
    create_provider_file_mapping(file_id, provider_id, Some(provider_file_id), serde_json::json!({})).await?;
    Ok(())
}

/// Load file content from storage
pub async fn load_file_content(file_id: Uuid) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Get file info using existing database function
    let file = get_file_by_id(file_id)
        .await?
        .ok_or("File not found")?;
    
    // Extract file extension from filename
    let extension = extract_extension(&file.filename);
    
    // Get the file path from file storage
    let file_path = FILE_STORAGE.get_original_path(file_id, &extension);
    
    // Read file content from storage
    let file_data = FILE_STORAGE.read_file_bytes(&file_path).await?;
    
    Ok(file_data)
}

/// Add provider mapping to database
pub async fn add_provider_mapping_to_file_ref(file_ref: &FileReference, provider_id: Uuid, provider_file_id: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use existing database function to create provider file mapping
    save_provider_file_mapping(file_ref.file_id, provider_id, provider_file_id).await?;
    Ok(())
}

/// Format file size for display
pub fn format_file_size(size_bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    
    if size_bytes >= GB {
        format!("{:.1} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.1} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.1} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", size_bytes)
    }
}