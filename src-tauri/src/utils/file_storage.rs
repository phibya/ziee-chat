use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;
use sha2::{Sha256, Digest};

pub struct FileStorage {
    base_path: PathBuf,
}

impl FileStorage {
    pub fn new(app_data_dir: &Path) -> Self {
        let base_path = app_data_dir.join("files");
        Self { base_path }
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create all required directories
        let directories = [
            &self.base_path,
            &self.base_path.join("originals"),
            &self.base_path.join("text"),
            &self.base_path.join("images"),
            &self.base_path.join("thumbnails"),
        ];

        for dir in directories {
            if !dir.exists() {
                tokio_fs::create_dir_all(dir).await?;
                println!("Created directory: {:?}", dir);
            }
        }

        Ok(())
    }

    pub fn get_original_path(&self, file_id: Uuid, extension: &str) -> PathBuf {
        self.base_path
            .join("originals")
            .join(format!("{}.{}", file_id, extension))
    }

    pub fn get_text_path(&self, file_id: Uuid) -> PathBuf {
        self.base_path
            .join("text")
            .join(format!("{}.txt", file_id))
    }


    pub fn get_image_dir(&self, file_id: Uuid) -> PathBuf {
        self.base_path
            .join("images")
            .join(file_id.to_string())
    }

    pub fn get_image_path(&self, file_id: Uuid, page: u32) -> PathBuf {
        self.get_image_dir(file_id)
            .join(format!("page_{}.jpg", page))
    }

    pub fn get_thumbnail_dir(&self, file_id: Uuid) -> PathBuf {
        self.base_path
            .join("thumbnails")
            .join(file_id.to_string())
    }

    pub fn get_thumbnail_path(&self, file_id: Uuid, page: u32) -> PathBuf {
        self.get_thumbnail_dir(file_id)
            .join(format!("page_{}.jpg", page))
    }

    pub async fn save_file_bytes(
        &self,
        file_path: &Path,
        data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            tokio_fs::create_dir_all(parent).await?;
        }

        let mut file = tokio_fs::File::create(file_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        Ok(())
    }

    pub async fn save_file_stream<R: AsyncReadExt + Unpin>(
        &self,
        file_path: &Path,
        mut reader: R,
    ) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            tokio_fs::create_dir_all(parent).await?;
        }

        let mut file = tokio_fs::File::create(file_path).await?;
        let bytes_written = tokio::io::copy(&mut reader, &mut file).await?;
        file.sync_all().await?;

        Ok(bytes_written)
    }

    pub async fn read_file_bytes(
        &self,
        file_path: &Path,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let mut file = tokio_fs::File::open(file_path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;
        Ok(buffer)
    }

    pub async fn read_file_string(
        &self,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let bytes = self.read_file_bytes(file_path).await?;
        let content = String::from_utf8(bytes)?;
        Ok(content)
    }

    pub async fn save_text_content(
        &self,
        file_id: Uuid,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let text_path = self.get_text_path(file_id);
        self.save_file_bytes(&text_path, content.as_bytes()).await
    }


    pub async fn read_text_content(
        &self,
        file_id: Uuid,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let text_path = self.get_text_path(file_id);
        if !text_path.exists() {
            return Ok(None);
        }
        
        match self.read_file_string(&text_path).await {
            Ok(content) => Ok(Some(content)),
            Err(_) => Ok(None),
        }
    }


    pub async fn calculate_checksum(
        &self,
        file_path: &Path,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let data = self.read_file_bytes(file_path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub async fn delete_file(
        &self,
        file_id: Uuid,
        extension: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Delete original file if extension is provided
        if let Some(ext) = extension {
            let original_path = self.get_original_path(file_id, ext);
            if original_path.exists() {
                tokio_fs::remove_file(original_path).await?;
            }
        }

        // Delete text content
        let text_path = self.get_text_path(file_id);
        if text_path.exists() {
            tokio_fs::remove_file(text_path).await?;
        }


        // Delete images directory
        let image_dir = self.get_image_dir(file_id);
        if image_dir.exists() {
            tokio_fs::remove_dir_all(image_dir).await?;
        }

        // Delete thumbnails directory
        let thumbnail_dir = self.get_thumbnail_dir(file_id);
        if thumbnail_dir.exists() {
            tokio_fs::remove_dir_all(thumbnail_dir).await?;
        }

        Ok(())
    }

    pub async fn create_image_directory(
        &self,
        file_id: Uuid,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let image_dir = self.get_image_dir(file_id);
        tokio_fs::create_dir_all(&image_dir).await?;
        Ok(image_dir)
    }

    pub async fn create_thumbnail_directory(
        &self,
        file_id: Uuid,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let thumbnail_dir = self.get_thumbnail_dir(file_id);
        tokio_fs::create_dir_all(&thumbnail_dir).await?;
        Ok(thumbnail_dir)
    }

    pub fn file_exists(&self, file_path: &Path) -> bool {
        file_path.exists()
    }

    pub async fn get_file_size(&self, file_path: &Path) -> Result<u64, std::io::Error> {
        let metadata = tokio_fs::metadata(file_path).await?;
        Ok(metadata.len())
    }
}

// Utility functions
pub fn extract_extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("bin")
        .to_lowercase()
}

pub fn get_mime_type_from_extension(extension: &str) -> Option<String> {
    match extension.to_lowercase().as_str() {
        "txt" => Some("text/plain".to_string()),
        "pdf" => Some("application/pdf".to_string()),
        "jpg" | "jpeg" => Some("image/jpeg".to_string()),
        "png" => Some("image/png".to_string()),
        "gif" => Some("image/gif".to_string()),
        "webp" => Some("image/webp".to_string()),
        "mp4" => Some("video/mp4".to_string()),
        "mov" => Some("video/quicktime".to_string()),
        "avi" => Some("video/x-msvideo".to_string()),
        "mp3" => Some("audio/mpeg".to_string()),
        "wav" => Some("audio/wav".to_string()),
        // Microsoft Office formats
        "doc" => Some("application/msword".to_string()),
        "docx" => Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string()),
        "xls" => Some("application/vnd.ms-excel".to_string()),
        "xlsx" => Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()),
        "ppt" => Some("application/vnd.ms-powerpoint".to_string()),
        "pptx" => Some("application/vnd.openxmlformats-officedocument.presentationml.presentation".to_string()),
        // Rich Text Format
        "rtf" => Some("application/rtf".to_string()),
        // OpenDocument formats
        "odt" => Some("application/vnd.oasis.opendocument.text".to_string()),
        "ods" => Some("application/vnd.oasis.opendocument.spreadsheet".to_string()),
        "odp" => Some("application/vnd.oasis.opendocument.presentation".to_string()),
        "zip" => Some("application/zip".to_string()),
        "tar" => Some("application/x-tar".to_string()),
        "gz" => Some("application/gzip".to_string()),
        "json" => Some("application/json".to_string()),
        "xml" => Some("application/xml".to_string()),
        "html" | "htm" => Some("text/html".to_string()),
        "css" => Some("text/css".to_string()),
        "js" => Some("application/javascript".to_string()),
        "md" => Some("text/markdown".to_string()),
        _ => None,
    }
}