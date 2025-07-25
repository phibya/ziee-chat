use crate::APP_DATA_DIR;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub path: String,
    pub model_type: String,
}

pub struct ModelUtils;

impl ModelUtils {
    pub fn get_model_absolute_path(model_path: &str) -> PathBuf {
        let path = PathBuf::from(model_path);
        if path.is_absolute() {
            path
        } else {
            crate::get_app_data_dir().join(model_path)
        }
    }

    pub fn model_exists(model_path: &str) -> bool {
        Self::get_model_absolute_path(model_path).exists()
    }

    pub fn get_model_size(model_path: &str) -> Result<u64, std::io::Error> {
        let metadata = std::fs::metadata(Self::get_model_absolute_path(model_path))?;
        Ok(metadata.len())
    }

    pub fn list_models(path: &str) -> Result<Vec<String>, std::io::Error> {
        let mut models = Vec::new();
        let dir = std::fs::read_dir(Self::get_model_absolute_path(path))?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    models.push(name.to_string());
                }
            }
        }

        Ok(models)
    }
}

pub struct ModelDiscovery;

impl ModelDiscovery {
    pub fn scan_models_directory(path: &str) -> Result<Vec<ModelConfig>, std::io::Error> {
        let mut models = Vec::new();
        let dir = std::fs::read_dir(ModelUtils::get_model_absolute_path(path))?;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    models.push(ModelConfig {
                        name: name.to_string(),
                        path: path.to_string_lossy().to_string(),
                        model_type: "unknown".to_string(),
                    });
                }
            }
        }

        Ok(models)
    }
}
