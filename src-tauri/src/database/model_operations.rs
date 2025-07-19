use crate::database::models::*;
use chrono::Utc;
use sqlx::{FromRow, Row};
use uuid::Uuid;

pub struct ModelOperations;

impl ModelOperations {
    /// Create default model settings
    fn default_model_settings() -> serde_json::Value {
        serde_json::json!({
            "verbose": false,
            "max_num_seqs": 256,
            "block_size": 32,
            "kvcache_mem_gpu": 4096,
            "kvcache_mem_cpu": 128,
            "record_conversation": false,
            "holding_time": 500,
            "multi_process": false,
            "log": false,
            "device_type": "cpu",
            "device_ids": []
        })
    }

    /// Create default model parameters
    fn default_model_parameters() -> serde_json::Value {
        serde_json::json!({
            "contextSize": 4096,
            "gpuLayers": -1,
            "temperature": 0.7,
            "topK": 40,
            "topP": 0.95,
            "minP": 0.05,
            "repeatLastN": 64,
            "repeatPenalty": 1.1,
            "presencePenalty": 0.0,
            "frequencyPenalty": 0.0
        })
    }

    /// Create default model capabilities
    fn default_model_capabilities() -> serde_json::Value {
        serde_json::json!({
            "vision": false,
            "audio": false,
            "tools": false,
            "code_interpreter": false
        })
    }

    /// Create a Candle model with required architecture and default settings
    pub async fn create_candle_model(
        request: &CreateModelRequest,
        architecture: &str, // Architecture is required for Candle models
    ) -> Result<Model, sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let model_id = Uuid::new_v4();
        let now = Utc::now();

        let capabilities = request
            .capabilities
            .clone()
            .unwrap_or_else(Self::default_model_capabilities);

        // Create model settings with architecture and defaults
        let mut model_settings = Self::default_model_settings();
        model_settings["architecture"] = serde_json::Value::String(architecture.to_string());

        let row = sqlx::query(
            r#"
            INSERT INTO models (
                id, provider_id, name, alias, description, 
                file_size_bytes, enabled, 
                is_deprecated, is_active, capabilities, parameters, 
                validation_status, settings, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
            ) RETURNING id, provider_id, name, alias, description, 
                       file_size_bytes, enabled, 
                       is_deprecated, is_active, capabilities, parameters, 
                       validation_status, validation_issues, settings, port, pid, created_at, updated_at
            "#,
        )
        .bind(model_id)
        .bind(&request.provider_id)
        .bind(&request.name)
        .bind(&request.alias)
        .bind(&request.description)
        .bind(0i64)
        .bind(request.enabled.unwrap_or(false))
        .bind(false)
        .bind(false)
        .bind(capabilities)
        .bind(Self::default_model_parameters())
        .bind("pending")
        .bind(model_settings) // Model settings with architecture
        .bind(now)
        .bind(now)
        .fetch_one(pool.as_ref())
        .await?;

        let model: Model = Model::from_row(&row)?;
        Ok(model)
    }

    /// Get model by ID
    pub async fn get_model_by_id(model_id: &Uuid) -> Result<Option<Model>, sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let model: Option<Model> = sqlx::query_as(
            "SELECT id, provider_id, name, alias, description, 
                    file_size_bytes, enabled, 
                    is_deprecated, is_active, capabilities, parameters, 
                    validation_status, validation_issues, settings, port, pid, created_at, updated_at
             FROM models WHERE id = $1",
        )
        .bind(model_id)
        .fetch_optional(pool.as_ref())
        .await?;

        Ok(model)
    }

    /// Update model validation status and issues
    pub async fn update_model_validation(
        model_id: &Uuid,
        validation_status: &str,
        validation_issues: Option<&Vec<String>>,
        file_size_bytes: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let issues_json = validation_issues
            .map(|issues| serde_json::to_value(issues).unwrap_or(serde_json::Value::Null))
            .unwrap_or(serde_json::Value::Null);

        sqlx::query(
            r#"
            UPDATE models 
            SET validation_status = $1, 
                validation_issues = $2, 
                file_size_bytes = COALESCE($3, file_size_bytes),
                updated_at = $4
            WHERE id = $5
            "#,
        )
        .bind(validation_status)
        .bind(issues_json)
        .bind(file_size_bytes)
        .bind(Utc::now())
        .bind(model_id)
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    /// Update model status (enabled/active)
    pub async fn update_model_status(
        model_id: &Uuid,
        enabled: Option<bool>,
        is_active: Option<bool>,
    ) -> Result<(), sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        sqlx::query(
            r#"
            UPDATE models 
            SET enabled = COALESCE($1, enabled),
                is_active = COALESCE($2, is_active),
                updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(enabled)
        .bind(is_active)
        .bind(Utc::now())
        .bind(model_id)
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    /// Create a model file record
    pub async fn create_model_file(
        model_id: &Uuid,
        filename: &str,
        file_path: &str,
        file_size_bytes: i64,
        file_type: &str,
    ) -> Result<ModelFile, sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let file_id = Uuid::new_v4();
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            INSERT INTO model_files (
                id, model_id, filename, file_path, file_size_bytes, 
                file_type, upload_status, uploaded_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8
            ) RETURNING id, model_id, filename, file_path, file_size_bytes, 
                       file_type, upload_status, uploaded_at
            "#,
        )
        .bind(file_id)
        .bind(model_id)
        .bind(filename)
        .bind(file_path)
        .bind(file_size_bytes)
        .bind(file_type)
        .bind("completed")
        .bind(now)
        .fetch_one(pool.as_ref())
        .await?;

        let file = ModelFile {
            id: row.get("id"),
            model_id: row.get("model_id"),
            filename: row.get("filename"),
            file_path: row.get("file_path"),
            file_size_bytes: row.get("file_size_bytes"),
            file_type: row.get("file_type"),
            upload_status: row.get("upload_status"),
            uploaded_at: row.get("uploaded_at"),
        };

        Ok(file)
    }

    /// Get files for a model
    pub async fn get_model_files(model_id: &Uuid) -> Result<Vec<ModelFile>, sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let rows = sqlx::query(
            "SELECT id, model_id, filename, file_path, file_size_bytes, 
                    file_type, upload_status, uploaded_at
             FROM model_files WHERE model_id = $1 ORDER BY uploaded_at ASC",
        )
        .bind(model_id)
        .fetch_all(pool.as_ref())
        .await?;

        let mut files = Vec::new();
        for row in rows {
            let file = ModelFile {
                id: row.get("id"),
                model_id: row.get("model_id"),
                filename: row.get("filename"),
                file_path: row.get("file_path"),
                file_size_bytes: row.get("file_size_bytes"),
                file_type: row.get("file_type"),
                upload_status: row.get("upload_status"),
                uploaded_at: row.get("uploaded_at"),
            };
            files.push(file);
        }

        Ok(files)
    }

    /// Get storage statistics for a provider
    pub async fn get_provider_storage_stats(
        provider_id: &Uuid,
    ) -> Result<ModelStorageInfo, sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_models,
                COALESCE(SUM(file_size_bytes), 0) as total_storage_bytes,
                COUNT(*) FILTER (WHERE is_active = true) as active,
                COUNT(*) FILTER (WHERE is_active = false) as inactive,
                COUNT(*) FILTER (WHERE is_deprecated = true) as deprecated,
                COUNT(*) FILTER (WHERE enabled = true) as enabled,
                COUNT(*) FILTER (WHERE enabled = false) as disabled
            FROM models 
            WHERE provider_id = $1
            "#,
        )
        .bind(provider_id)
        .fetch_one(pool.as_ref())
        .await?;

        Ok(ModelStorageInfo {
            provider_id: *provider_id,
            total_models: row.get("total_models"),
            total_storage_bytes: row.get::<i64, _>("total_storage_bytes") as u64,
            models_by_status: ModelStatusCounts {
                active: row.get("active"),
                inactive: row.get("inactive"),
                deprecated: row.get("deprecated"),
                enabled: row.get("enabled"),
                disabled: row.get("disabled"),
            },
        })
    }

    /// Get all models with their files for full response
    pub async fn get_model_with_files(model_id: &Uuid) -> Result<Option<Model>, sqlx::Error> {
        let model = Self::get_model_by_id(model_id).await?;

        if let Some(model) = model {
            let files = Self::get_model_files(model_id).await?;
            Ok(Some(model.with_files(Some(files))))
        } else {
            Ok(None)
        }
    }

    /// Update model runtime information (PID and port)
    pub async fn update_model_runtime_info(
        model_id: &Uuid,
        pid: Option<i32>,
        port: Option<i32>,
        is_active: bool,
    ) -> Result<(), sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        sqlx::query(
            r#"
            UPDATE models 
            SET pid = $2, port = $3, is_active = $4, updated_at = $5
            WHERE id = $1
            "#,
        )
        .bind(model_id)
        .bind(pid)
        .bind(port)
        .bind(is_active)
        .bind(Utc::now())
        .execute(pool.as_ref())
        .await?;

        Ok(())
    }

    /// Get model runtime information by model ID
    pub async fn get_model_runtime_info(
        model_id: &Uuid,
    ) -> Result<Option<(i32, i32)>, sqlx::Error> {
        let pool = crate::database::get_database_pool()?;
        let row = sqlx::query(
            r#"
            SELECT pid, port
            FROM models
            WHERE id = $1
            "#,
        )
        .bind(model_id)
        .fetch_optional(pool.as_ref())
        .await?;

        if let Some(row) = row {
            if let (Some(pid), Some(port)) = (
                row.get::<Option<i32>, _>("pid"),
                row.get::<Option<i32>, _>("port"),
            ) {
                return Ok(Some((pid, port)));
            }
        }

        Ok(None)
    }
}
