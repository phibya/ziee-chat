use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

pub struct RagFileStorage {
    base_path: PathBuf,
}

impl RagFileStorage {
    pub fn new(app_data_dir: &Path) -> Self {
        let base_path = app_data_dir.join("rag-files");
        Self { base_path }
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.base_path.exists() {
            tokio_fs::create_dir_all(&self.base_path).await?;
            println!("Created RAG files directory: {:?}", self.base_path);
        }
        Ok(())
    }

    pub fn get_instance_dir(&self, instance_id: Uuid) -> PathBuf {
        self.base_path.join(instance_id.to_string())
    }

    pub fn get_file_path(&self, instance_id: Uuid, file_id: Uuid, extension: &str) -> PathBuf {
        self.get_instance_dir(instance_id)
            .join(format!("{}.{}", file_id, extension))
    }

    pub async fn save_rag_file(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        extension: &str,
        data: &[u8],
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let instance_dir = self.get_instance_dir(instance_id);
        tokio_fs::create_dir_all(&instance_dir).await?;

        let file_path = self.get_file_path(instance_id, file_id, extension);
        let mut file = tokio_fs::File::create(&file_path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;

        Ok(file_path)
    }

    pub async fn delete_instance_files(
        &self,
        instance_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let instance_dir = self.get_instance_dir(instance_id);
        if instance_dir.exists() {
            tokio_fs::remove_dir_all(instance_dir).await?;
        }
        Ok(())
    }

    pub async fn delete_rag_file(
        &self,
        instance_id: Uuid,
        file_id: Uuid,
        extension: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let file_path = self.get_file_path(instance_id, file_id, extension);
        if file_path.exists() {
            tokio_fs::remove_file(file_path).await?;
        }
        Ok(())
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
}
