use crate::ai::core::providers::FileReference;
use crate::database::queries::files::*;
use crate::global::FILE_STORAGE;
use crate::utils::file_storage::extract_extension;
use base64::Engine;
use std::collections::HashMap;
use uuid::Uuid;

/// Load FileReference from database
pub async fn load_file_reference(
    file_id: Uuid,
) -> Result<FileReference, Box<dyn std::error::Error + Send + Sync>> {
    // Get file info using existing database function
    let file = get_file_by_id(file_id).await?.ok_or("File not found")?;

    Ok(FileReference {
        file_id: file.id,
        filename: file.filename,
        file_size: file.file_size,
        mime_type: file.mime_type,
        checksum: file.checksum,
    })
}

/// Get provider file mappings for a file
pub async fn get_provider_file_mappings(
    file_id: Uuid,
) -> Result<HashMap<Uuid, String>, Box<dyn std::error::Error + Send + Sync>> {
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
pub async fn save_provider_file_mapping(
    file_id: Uuid,
    provider_id: Uuid,
    provider_file_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use existing database function to create provider file mapping
    create_provider_file_mapping(
        file_id,
        provider_id,
        Some(provider_file_id),
        serde_json::json!({}),
    )
    .await?;
    Ok(())
}

/// Load file content from storage
pub async fn load_file_content(
    file_id: Uuid,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    // Get file info using existing database function
    let file = get_file_by_id(file_id).await?.ok_or("File not found")?;

    // Extract file extension from filename
    let extension = extract_extension(&file.filename);

    // Get the file path from file storage
    let file_path = FILE_STORAGE.get_original_path(file_id, &extension);

    // Read file content from storage
    let file_data = FILE_STORAGE.read_file_bytes(&file_path).await?;

    Ok(file_data)
}

/// Add provider mapping to database
pub async fn add_provider_mapping_to_file_ref(
    file_ref: &FileReference,
    provider_id: Uuid,
    provider_file_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Use existing database function to create provider file mapping
    save_provider_file_mapping(file_ref.file_id, provider_id, provider_file_id).await?;
    Ok(())
}

/// Load text content for a file reference
pub async fn load_text_content(
    file_id: Uuid,
) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
    FILE_STORAGE.read_text_content(file_id).await
}

/// Load image data for a file reference as base64
pub async fn load_image_as_base64(
    file_id: Uuid,
    filename: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let extension = extract_extension(filename);
    let file_path = FILE_STORAGE.get_original_path(file_id, &extension);
    let file_bytes = FILE_STORAGE.read_file_bytes(&file_path).await?;

    // Get MIME type from extension
    let mime_type = crate::utils::file_storage::get_mime_type_from_extension(&extension)
        .unwrap_or_else(|| "image/jpeg".to_string());

    let base64_content = base64::engine::general_purpose::STANDARD.encode(&file_bytes);
    Ok(format!("data:{};base64,{}", mime_type, base64_content))
}

/// Load all page images for a document/PDF as base64 strings
pub async fn load_document_images_as_base64(
    file_id: Uuid,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let image_dir = FILE_STORAGE.get_image_dir(file_id);
    let mut images = Vec::new();

    if !image_dir.exists() {
        return Ok(images);
    }

    // Read directory to find all page images
    let mut entries = tokio::fs::read_dir(&image_dir).await?;
    let mut page_paths = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with("page_") && filename.ends_with(".jpg") {
                page_paths.push(path);
            }
        }
    }

    // Sort paths by page number
    page_paths.sort_by(|a, b| {
        let extract_page_num = |path: &std::path::Path| -> u32 {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .and_then(|s| s.strip_prefix("page_"))
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0)
        };
        extract_page_num(a).cmp(&extract_page_num(b))
    });

    // Convert each image to base64
    for path in page_paths {
        if let Ok(bytes) = FILE_STORAGE.read_file_bytes(&path).await {
            let base64_content = base64::engine::general_purpose::STANDARD.encode(&bytes);
            images.push(format!("data:image/jpeg;base64,{}", base64_content));
        }
    }

    Ok(images)
}

/// Get comprehensive file content based on file type for local provider
pub async fn get_file_content_for_local_provider(
    file_ref: &FileReference,
    supports_vision: bool,
) -> Result<LocalProviderFileContent, Box<dyn std::error::Error + Send + Sync>> {
    // For images, return base64 if vision is supported
    if file_ref.is_image() && supports_vision {
        let base64_data = load_image_as_base64(file_ref.file_id, &file_ref.filename).await?;
        return Ok(LocalProviderFileContent::ImageBase64(base64_data));
    }

    // For text files, return text content only
    if file_ref.is_text() {
        if let Ok(Some(text_content)) = load_text_content(file_ref.file_id).await {
            return Ok(LocalProviderFileContent::TextOnly(text_content));
        }

        // Fallback: try to read raw file content if it's a text file
        let extension = extract_extension(&file_ref.filename);
        let file_path = FILE_STORAGE.get_original_path(file_ref.file_id, &extension);

        if let Ok(file_bytes) = FILE_STORAGE.read_file_bytes(&file_path).await {
            if let Ok(raw_text) = String::from_utf8(file_bytes) {
                return Ok(LocalProviderFileContent::TextOnly(raw_text));
            }
        }

        return Err("Could not read text file content".into());
    }

    // For spreadsheets, return text content only
    if file_ref.is_spreadsheet() {
        if let Ok(Some(text_content)) = load_text_content(file_ref.file_id).await {
            return Ok(LocalProviderFileContent::TextOnly(text_content));
        }
        return Err("Could not read spreadsheet text content".into());
    }

    // For PDFs and documents, return both text and images
    if file_ref.is_pdf() || file_ref.is_document() {
        let text_content = load_text_content(file_ref.file_id)
            .await?
            .unwrap_or_default();
        let images = load_document_images_as_base64(file_ref.file_id).await?;

        return Ok(LocalProviderFileContent::TextAndImages {
            text: text_content,
            images,
        });
    }

    // For other file types, return basic file info
    Ok(LocalProviderFileContent::FileInfo {
        filename: file_ref.filename.clone(),
        size: file_ref.file_size,
        mime_type: file_ref.mime_type.clone(),
    })
}

/// Local provider specific file content enum
#[derive(Debug, Clone)]
pub enum LocalProviderFileContent {
    /// Image file as base64 string (for vision models)
    ImageBase64(String),
    /// Text-only content
    TextOnly(String),
    /// Combined text and images (for documents/PDFs)
    TextAndImages { text: String, images: Vec<String> },
    /// Basic file information (for unsupported types)
    FileInfo {
        filename: String,
        size: i64,
        mime_type: Option<String>,
    },
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
