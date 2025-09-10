use crate::database::{
    get_database_pool,
    models::{
        CreateRAGInstanceRequest, CreateSystemRAGInstanceRequest, RAGInstance,
        RAGInstanceListResponse, UpdateRAGInstanceRequest, RAGInstanceErrorCode,
        RAGProcessingStatus,
        rag_instance,
    },
    queries::user_group_rag_providers::can_user_create_rag_instance,
};
use uuid::Uuid;

/// Create a user RAG instance (is_system = false)
pub async fn create_user_rag_instance(
    user_id: Uuid,
    request: CreateRAGInstanceRequest,
) -> Result<RAGInstance, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First validate that user can create instances with this provider
    let can_create = can_user_create_rag_instance(user_id, request.provider_id).await?;
    if !can_create {
        eprintln!(
            "User {} cannot create instances with provider {}",
            user_id, request.provider_id
        );
        return Err(sqlx::Error::RowNotFound);
    }

    let instance_id = Uuid::new_v4();
    let engine_type_str = request.engine_type.as_str();

    // Serialize consolidated engine settings for database storage
    let engine_settings_json = serde_json::to_value(&request.engine_settings.as_ref().cloned().unwrap_or_default())
        .unwrap_or_else(|_| serde_json::json!({}));

    let instance = sqlx::query_as!(
        RAGInstance,
        r#"INSERT INTO rag_instances (
            id, provider_id, user_id, project_id, name, display_name, description,
            enabled, is_active, is_system, engine_type, 
            engine_settings, embedding_model_id, llm_model_id, parameters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, provider_id, user_id, project_id, name, display_name, description,
                  enabled, is_active, is_system, status, error_code,
                  engine_type,
                  engine_settings,
                  embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at"#,
        instance_id,
        request.provider_id,
        user_id,
        request.project_id,
        &request.name,
        &request.display_name,
        request.description.as_deref(),
        true,  // enabled = true by default
        false, // is_active = false by default
        false, // is_system = false for user instances
        engine_type_str,
        &engine_settings_json,
        request.embedding_model_id,
        request.llm_model_id,
        request.parameters.unwrap_or_else(|| serde_json::json!({}))
    )
    .fetch_one(pool)
    .await?;

    // Reset instance state to ensure clean starting point
    if let Err(e) = reset_rag_instance_state(instance_id).await {
        tracing::warn!("Failed to reset state for new RAG instance {}: {}", instance_id, e);
    }

    Ok(instance)
}

/// Create a system RAG instance (is_system = true, admin only)
pub async fn create_system_rag_instance(
    request: CreateSystemRAGInstanceRequest,
) -> Result<RAGInstance, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let instance_id = Uuid::new_v4();
    let engine_type_str = request.engine_type.as_str();

    // Serialize consolidated engine settings for database storage
    let engine_settings_json = serde_json::to_value(&request.engine_settings.as_ref().cloned().unwrap_or_default())
        .unwrap_or_else(|_| serde_json::json!({}));

    let instance = sqlx::query_as!(
        RAGInstance,
        r#"INSERT INTO rag_instances (
            id, provider_id, user_id, project_id, name, display_name, description,
            enabled, is_active, is_system, engine_type, 
            engine_settings, embedding_model_id, llm_model_id, parameters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, provider_id, user_id, project_id, name, display_name, description,
                  enabled, is_active, is_system, status, error_code,
                  engine_type, 
                  engine_settings,
                  embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at"#,
        instance_id,
        request.provider_id,
        None::<Uuid>, // user_id = null for system instances
        None::<Uuid>, // project_id = null for system instances
        &request.name,
        &request.display_name,
        request.description.as_deref(),
        true,  // enabled = true by default
        false, // is_active = false by default
        true,  // is_system = true for system instances
        engine_type_str,
        &engine_settings_json,
        request.embedding_model_id,
        request.llm_model_id,
        request.parameters.unwrap_or_else(|| serde_json::json!({}))
    )
    .fetch_one(pool)
    .await?;

    // Reset instance state to ensure clean starting point
    if let Err(e) = reset_rag_instance_state(instance_id).await {
        tracing::warn!("Failed to reset state for new system RAG instance {}: {}", instance_id, e);
    }

    Ok(instance)
}

/// Get RAG instance by ID (active instances for regular users, all instances for admins)
pub async fn get_rag_instance(
    instance_id: Uuid,
    user_id: Uuid,
) -> Result<Option<RAGInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if user has admin read permission
    use crate::database::queries::users::get_user_by_id;
    let has_admin_read = if let Some(user) = get_user_by_id(user_id).await? {
        use crate::api::permissions::check_permission;
        check_permission(&user, "rag::admin::instances::read")
    } else {
        false
    };

    let instance: Option<RAGInstance> = if has_admin_read {
        // Admin users can see all instances (enabled and disabled)
        sqlx::query_as!(
            RAGInstance,
            r#"SELECT id, provider_id, user_id, project_id, name, display_name, description,
                    enabled, is_active, is_system, status, error_code,
                    engine_type, 
                    engine_settings,
                    embedding_model_id, llm_model_id, age_graph_name, parameters, 
                    created_at, updated_at
             FROM rag_instances 
             WHERE id = $1"#,
            instance_id
        )
        .fetch_optional(pool)
        .await?
    } else {
        // Regular users can only see enabled instances
        sqlx::query_as!(
            RAGInstance,
            r#"SELECT id, provider_id, user_id, project_id, name, display_name, description,
                    enabled, is_active, is_system, status, error_code,
                    engine_type, 
                    engine_settings,
                    embedding_model_id, llm_model_id, age_graph_name, parameters, 
                    created_at, updated_at
             FROM rag_instances 
             WHERE id = $1 AND enabled = true"#,
            instance_id
        )
        .fetch_optional(pool)
        .await?
    };

    Ok(instance)
}

/// List user's RAG instances with optional system instances
pub async fn list_user_rag_instances(
    user_id: Uuid,
    page: i32,
    per_page: i32,
    include_system: Option<bool>,
) -> Result<RAGInstanceListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;
    let include_sys = include_system.unwrap_or(false);

    let (instances, total_count) = if include_sys {
        // Include both user instances AND accessible system instances (only enabled ones)
        let total = sqlx::query_scalar!(
            "SELECT COUNT(DISTINCT ri.id) FROM rag_instances ri
             LEFT JOIN user_group_rag_providers ugrp ON ri.provider_id = ugrp.provider_id AND ri.is_system = true
             LEFT JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id AND ugm.user_id = $1
             LEFT JOIN user_groups ug ON ugm.group_id = ug.id AND ug.is_active = true
             WHERE ((ri.user_id = $1 AND ri.is_system = false AND ri.enabled = true) 
             OR (ri.is_system = true AND ri.enabled = true AND ugm.user_id IS NOT NULL))",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // Complex JOIN query with DISTINCT and multiple JOINs
        let instances = sqlx::query_as!(
            RAGInstance,
            r#"SELECT DISTINCT ri.id, ri.provider_id, ri.user_id, ri.project_id, ri.name, ri.display_name, ri.description,
                    ri.enabled, ri.is_active, ri.is_system, ri.status, ri.error_code,
                    ri.engine_type, 
                    ri.engine_settings,
                    ri.embedding_model_id, ri.llm_model_id, ri.age_graph_name, ri.parameters, 
                    ri.created_at, ri.updated_at
             FROM rag_instances ri
             LEFT JOIN user_group_rag_providers ugrp ON ri.provider_id = ugrp.provider_id AND ri.is_system = true
             LEFT JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id AND ugm.user_id = $1
             LEFT JOIN user_groups ug ON ugm.group_id = ug.id AND ug.is_active = true
             WHERE ((ri.user_id = $1 AND ri.is_system = false AND ri.enabled = true) 
             OR (ri.is_system = true AND ri.enabled = true AND ugm.user_id IS NOT NULL))
             ORDER BY ri.is_system ASC, ri.created_at DESC 
             LIMIT $2 OFFSET $3"#,
            user_id,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        (instances, total)
    } else {
        // Only user instances (only enabled ones)
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM rag_instances WHERE user_id = $1 AND is_system = false AND enabled = true",
            user_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let instances = sqlx::query_as!(
            RAGInstance,
            r#"SELECT id, provider_id, user_id, project_id, name, display_name, description,
                    enabled, is_active, is_system, status, error_code,
                    engine_type, 
                    engine_settings,
                    embedding_model_id, llm_model_id, age_graph_name, parameters, 
                    created_at, updated_at
             FROM rag_instances 
             WHERE user_id = $1 AND is_system = false AND enabled = true
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3"#,
            user_id,
            per_page as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        (instances, total)
    };

    Ok(RAGInstanceListResponse {
        instances,
        total: total_count,
        page,
        per_page,
    })
}

/// List system RAG instances (is_system = true, admin only)
pub async fn list_system_rag_instances(
    page: i32,
    per_page: i32,
) -> Result<RAGInstanceListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    // Get total count of system instances
    let total_count =
        sqlx::query_scalar!("SELECT COUNT(*) FROM rag_instances WHERE is_system = true")
            .fetch_one(pool)
            .await?
            .unwrap_or(0);

    // Get system instances with pagination
    let instances = sqlx::query_as!(
        RAGInstance,
        r#"SELECT id, provider_id, user_id, project_id, name, display_name, description,
                enabled, is_active, is_system, status, error_code,
                engine_type, 
                engine_settings,
                embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances 
         WHERE is_system = true
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2"#,
        per_page as i64,
        offset as i64
    )
    .fetch_all(pool)
    .await?;

    Ok(RAGInstanceListResponse {
        instances,
        total: total_count,
        page,
        per_page,
    })
}

/// Update RAG instance with ownership validation
pub async fn update_rag_instance(
    instance_id: Uuid,
    request: UpdateRAGInstanceRequest,
) -> Result<Option<RAGInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Get current instance to check for changes that require state reset
    let current_instance = get_rag_instance_by_id(instance_id).await?;
    let should_reset_state = if let Some(current) = &current_instance {
        // Check if engine type is changing
        let engine_type_changed = request.engine_type.as_ref()
            .map(|new_type| new_type != &current.engine_type)
            .unwrap_or(false);
        
        // Check if engine settings are changing
        let engine_settings_changed = request.engine_settings.as_ref()
            .map(|new_settings| {
                let new_json = serde_json::to_value(new_settings).unwrap_or_else(|_| serde_json::json!({}));
                let current_json = serde_json::to_value(&current.engine_settings).unwrap_or_else(|_| serde_json::json!({}));
                new_json != current_json
            })
            .unwrap_or(false);

        // Check if embedding model is changing (only if provided in request)
        let embedding_model_changed = request.embedding_model_id
            .map(|new_id| Some(new_id) != current.embedding_model_id)
            .unwrap_or(false);

        // Check if LLM model is changing (only if provided in request)
        let llm_model_changed = request.llm_model_id
            .map(|new_id| Some(new_id) != current.llm_model_id)
            .unwrap_or(false);

        engine_type_changed || engine_settings_changed || embedding_model_changed || llm_model_changed
    } else {
        false // Instance doesn't exist, no need to reset
    };

    // Handle engine settings update
    let engine_settings_update = if let Some(settings) = &request.engine_settings {
        Some(serde_json::to_value(settings).unwrap_or_else(|_| serde_json::json!({})))
    } else {
        None
    };

    // Replace COALESCE with separate conditional updates
    if let Some(name) = &request.name {
        sqlx::query!(
            "UPDATE rag_instances SET name = $1, updated_at = NOW() WHERE id = $2",
            name,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(description) = &request.description {
        sqlx::query!(
            "UPDATE rag_instances SET description = $1, updated_at = NOW() WHERE id = $2",
            description,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(enabled) = request.enabled {
        sqlx::query!(
            "UPDATE rag_instances SET enabled = $1, updated_at = NOW() WHERE id = $2",
            enabled,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(is_active) = request.is_active {
        sqlx::query!(
            "UPDATE rag_instances SET is_active = $1, updated_at = NOW() WHERE id = $2",
            is_active,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(engine_type) = &request.engine_type {
        let engine_type_str = engine_type.as_str();
        sqlx::query!(
            "UPDATE rag_instances SET engine_type = $1, updated_at = NOW() WHERE id = $2",
            engine_type_str,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(embedding_model_id) = request.embedding_model_id {
        sqlx::query!(
            "UPDATE rag_instances SET embedding_model_id = $1, updated_at = NOW() WHERE id = $2",
            embedding_model_id,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(llm_model_id) = request.llm_model_id {
        sqlx::query!(
            "UPDATE rag_instances SET llm_model_id = $1, updated_at = NOW() WHERE id = $2",
            llm_model_id,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(parameters) = &request.parameters {
        sqlx::query!(
            "UPDATE rag_instances SET parameters = $1, updated_at = NOW() WHERE id = $2",
            parameters,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(error_code) = &request.error_code {
        let error_code_str = error_code.as_str();
        sqlx::query!(
            "UPDATE rag_instances SET error_code = $1, updated_at = NOW() WHERE id = $2",
            error_code_str,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    if let Some(engine_settings) = &engine_settings_update {
        sqlx::query!(
            "UPDATE rag_instances SET engine_settings = $1, updated_at = NOW() WHERE id = $2",
            engine_settings,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    // Reset instance state if configuration changes require it
    if should_reset_state {
        if let Err(e) = reset_rag_instance_state(instance_id).await {
            tracing::warn!("Failed to reset state for updated RAG instance {}: {}", instance_id, e);
        } else {
            tracing::info!("Reset state for RAG instance {} due to configuration changes", instance_id);
        }
    }

    // Return the updated instance
    let instance = sqlx::query_as!(
        RAGInstance,
        r#"SELECT id, provider_id, user_id, project_id, name, display_name, description,
                  enabled, is_active, is_system, status, error_code,
                  engine_type, 
                  engine_settings,
                  embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at
         FROM rag_instances 
         WHERE id = $1"#,
        instance_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(instance)
}

/// Delete RAG instance with automatic CASCADE cleanup
pub async fn delete_rag_instance(instance_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Drop partition (this instantly removes all vector documents and indexes)
    if let Err(e) = drop_partition_for_instance(instance_id).await {
        tracing::warn!("Failed to drop partition for RAG instance {} during deletion: {}", instance_id, e);
    }

    // Delete RAG instance (CASCADE will automatically delete associated files and rag_instance_files)
    let result = sqlx::query!("DELETE FROM rag_instances WHERE id = $1", instance_id)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected() > 0;

    if deleted {
        tracing::info!("Successfully deleted RAG instance {}", instance_id);
        
        // Clean up file system
        if let Err(e) = crate::global::RAG_FILE_STORAGE
            .delete_instance_files(instance_id)
            .await
        {
            tracing::error!("Failed to delete RAG instance files from filesystem: {}", e);
        }
    }

    Ok(deleted)
}

/// Validate user can access RAG instance
/// - Users can access their own instances
/// - Users can view system instances if they have provider access
/// - Users can edit system instances if they have RagAdminInstancesEdit permission
pub async fn validate_rag_instance_access(
    user_id: Uuid,
    instance_id: Uuid,
    require_owner: bool,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // First get the instance and user details
    let query_result = sqlx::query!(
        "SELECT id, user_id, is_system, provider_id
         FROM rag_instances 
         WHERE id = $1",
        instance_id
    )
    .fetch_optional(pool)
    .await?;

    let (instance_user_id, is_system, provider_id) = match query_result {
        Some(result) => (result.user_id, result.is_system, result.provider_id),
        None => return Ok(false), // Instance not found
    };

    // If user owns the instance, they have access
    if instance_user_id == Some(user_id) {
        return Ok(true);
    }

    // If not a system instance, user can't access
    if !is_system {
        return Ok(false);
    }

    // For system instances, check provider access first
    let has_provider_access = sqlx::query_scalar!(
        "SELECT true FROM user_group_rag_providers ugrp
         JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id
         JOIN user_groups ug ON ugm.group_id = ug.id
         WHERE ugm.user_id = $1 AND ug.is_active = true 
         AND ugrp.provider_id = $2
         LIMIT 1",
        user_id,
        provider_id
    )
    .fetch_optional(pool)
    .await?;

    // If no provider access, deny access
    if has_provider_access.is_none() {
        return Ok(false);
    }

    // If only read access needed, grant it
    if !require_owner {
        return Ok(true);
    }

    // For write operations on system instances, check admin permission
    // Use existing function to get user with all related data including groups
    use crate::database::queries::users::get_user_by_id;

    if let Some(user) = get_user_by_id(user_id).await? {
        // Check if user has admin edit permission using the permission system
        use crate::api::permissions::check_permission;
        Ok(check_permission(&user, "rag::admin::instances::edit"))
    } else {
        Ok(false)
    }
}

/// Get RAG instance by ID without user permission checking (for internal use)
pub async fn get_rag_instance_by_id(instance_id: Uuid) -> Result<Option<RAGInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let instance = sqlx::query_as!(
        RAGInstance,
        r#"SELECT id, provider_id, user_id, project_id, name, display_name, description, enabled, is_active, is_system, status, error_code,
                engine_type, 
                engine_settings,
                embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances WHERE id = $1"#,
        instance_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(instance)
}

/// Get RAG instance status with file statistics for streaming
/// Returns None if no changes since the provided timestamp
pub async fn get_rag_instance_status_with_stats(
    instance_id: Uuid,
    since: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Option<RAGInstanceWithStats>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if instance has been updated since timestamp
    if let Some(since_time) = since {
        let has_updates = sqlx::query_scalar!(
            "SELECT true FROM rag_instances 
             WHERE id = $1 AND updated_at > $2",
            instance_id,
            since_time
        )
        .fetch_optional(pool)
        .await?;

        if has_updates.is_none() {
            return Ok(None);
        }
    }

    let result = sqlx::query_as!(
        RAGInstanceWithStats,
        r#"SELECT 
            ri.id, ri.name, ri.enabled, ri.is_active, ri.error_code, ri.updated_at,
            COALESCE(file_stats.total_files, 0) as "total_files!",
            COALESCE(file_stats.processed_files, 0) as "processed_files!",
            COALESCE(file_stats.failed_files, 0) as "failed_files!",
            COALESCE(file_stats.processing_files, 0) as "processing_files!"
         FROM rag_instances ri
         LEFT JOIN (
            SELECT 
                rag_instance_id,
                COUNT(*) as total_files,
                COUNT(CASE WHEN processing_status = 'completed' THEN 1 END) as processed_files,
                COUNT(CASE WHEN processing_status = 'failed' THEN 1 END) as failed_files,
                COUNT(CASE WHEN processing_status = 'processing' THEN 1 END) as processing_files
            FROM rag_instance_files 
            WHERE rag_instance_id = $1
            GROUP BY rag_instance_id
         ) file_stats ON ri.id = file_stats.rag_instance_id
         WHERE ri.id = $1"#,
        instance_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result)
}

/// Get file processing status details for a RAG instance
/// Returns only files that have been updated since the provided timestamp
pub async fn get_instance_file_processing_details(
    instance_id: Uuid,
    since: Option<chrono::DateTime<chrono::Utc>>,
) -> Result<Vec<RAGFileProcessingDetail>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    if let Some(since_time) = since {
        let files = sqlx::query_as!(
            RAGFileProcessingDetail,
            r#"SELECT 
                rif.file_id, f.filename, rif.processing_status, 
                NULL as current_stage, rif.processing_error, 
                rif.created_at as processing_started_at
             FROM rag_instance_files rif
             JOIN files f ON rif.file_id = f.id
             WHERE rif.rag_instance_id = $1 AND rif.updated_at > $2
             ORDER BY rif.updated_at DESC"#,
            instance_id,
            since_time
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    } else {
        let files = sqlx::query_as!(
            RAGFileProcessingDetail,
            r#"SELECT 
                rif.file_id, f.filename, rif.processing_status, 
                NULL as current_stage, rif.processing_error, 
                rif.created_at as processing_started_at
             FROM rag_instance_files rif
             JOIN files f ON rif.file_id = f.id
             WHERE rif.rag_instance_id = $1
             ORDER BY rif.updated_at DESC"#,
            instance_id
        )
        .fetch_all(pool)
        .await?;

        Ok(files)
    }
}

// Supporting structs for status queries
#[derive(Debug, Clone)]
pub struct RAGInstanceWithStats {
    pub id: Uuid,
    pub name: String,
    pub enabled: bool,
    pub is_active: bool,
    pub error_code: crate::database::types::EnumOption<RAGInstanceErrorCode>,
    pub total_files: i64,
    pub processed_files: i64,
    pub failed_files: i64,
    pub processing_files: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct RAGFileProcessingDetail {
    pub file_id: Uuid,
    pub filename: String,
    pub processing_status: String,
    pub current_stage: Option<String>,
    pub processing_error: Option<String>,
    pub processing_started_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ============================================================================
// Index Lifecycle Management Functions
// ============================================================================


/// Reset all file processing status to pending for a RAG instance
pub async fn reset_processing_status(instance_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let updated_count = sqlx::query!(
        r#"
        UPDATE rag_instance_files 
        SET processing_status = $2, 
            processing_error = NULL,
            processed_at = NULL,
            updated_at = NOW()
        WHERE rag_instance_id = $1
        "#,
        instance_id,
        RAGProcessingStatus::Pending.as_str()
    )
    .execute(pool)
    .await?
    .rows_affected();

    tracing::info!(
        "Reset processing status to pending for {} files in instance {}",
        updated_count,
        instance_id
    );

    Ok(())
}

/// Clear all processing pipeline records for a RAG instance
pub async fn clear_processing_pipeline(instance_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let deleted_count = sqlx::query!(
        "DELETE FROM rag_processing_pipeline WHERE rag_instance_id = $1",
        instance_id
    )
    .execute(pool)
    .await?
    .rows_affected();

    tracing::info!(
        "Cleared {} processing pipeline records for instance {}",
        deleted_count,
        instance_id
    );

    Ok(())
}


/// Create a partition for a specific RAG instance
async fn create_partition_for_instance(instance_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let instance_id_str = instance_id.to_string().replace("-", "_");
    let partition_name = format!("simple_vector_documents_{}", instance_id_str);

    let create_partition_query = format!(
        r#"
        CREATE TABLE IF NOT EXISTS {}
        PARTITION OF simple_vector_documents
        FOR VALUES IN ('{}')
        "#,
        partition_name, instance_id
    );

    sqlx::query(&create_partition_query)
        .execute(pool)
        .await?;

    tracing::info!("Created partition {} for RAG instance {}", partition_name, instance_id);
    Ok(())
}


/// Drop partition for a specific RAG instance
async fn drop_partition_for_instance(instance_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let instance_id_str = instance_id.to_string().replace("-", "_");
    let partition_name = format!("simple_vector_documents_{}", instance_id_str);

    let drop_partition_query = format!(
        r#"
        DROP TABLE IF EXISTS {}
        "#,
        partition_name
    );

    sqlx::query(&drop_partition_query)
        .execute(pool)
        .await?;

    tracing::info!("Dropped partition {} for RAG instance {}", partition_name, instance_id);
    Ok(())
}


/// Create HNSW index for a RAG instance on its specific partition
pub async fn create_rag_instance_index(instance_id: Uuid) -> Result<(), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let instance_id_str = instance_id.to_string().replace("-", "_");
    let partition_name = format!("simple_vector_documents_{}", instance_id_str);
    let index_name = format!("idx_simple_vector_docs_embedding_{}", instance_id_str);

    // Get RAG instance to find embedding model
    let instance = get_rag_instance_by_id(instance_id).await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;
    
    // Get embedding model ID
    let embedding_model_id = instance.embedding_model_id
        .ok_or_else(|| {
            tracing::warn!("No embedding model configured for RAG instance {}", instance_id);
            sqlx::Error::RowNotFound
        })?;
    
    // Get dimensions from AIModel
    let dimensions = match crate::ai::model_manager::model_factory::create_ai_model(embedding_model_id).await {
        Ok(ai_model) => {
            match ai_model.get_embedding_dimension().await {
                Some(dim) => dim,
                None => {
                    tracing::error!("Could not get dimensions from model {}, marking instance as failed", embedding_model_id);
                    
                    // Update RAG instance status to error
                    let _ = sqlx::query!(
                        r#"
                        UPDATE rag_instances 
                        SET status = $1, is_active = $2, error_code = $3, embedding_model_id = $4
                        WHERE id = $5
                        "#,
                        rag_instance::RAGInstanceStatus::Error.as_str(),
                        false,
                        rag_instance::RAGInstanceErrorCode::EmbeddingModelTestFailed.as_str(),
                        None::<Uuid>,
                        instance_id
                    )
                    .execute(pool)
                    .await;
                    
                    return Err(sqlx::Error::RowNotFound);
                }
            }
        }
        Err(e) => {
            tracing::error!("Could not create AI model {}: {}, marking instance as failed", embedding_model_id, e);
            
            // Update RAG instance status to error
            let _ = sqlx::query!(
                r#"
                UPDATE rag_instances 
                SET status = $1, is_active = $2, error_code = $3, embedding_model_id = $4
                WHERE id = $5
                "#,
                rag_instance::RAGInstanceStatus::Error.as_str(),
                false,
                rag_instance::RAGInstanceErrorCode::EmbeddingModelTestFailed.as_str(),
                None::<Uuid>,
                instance_id
            )
            .execute(pool)
            .await;
            
            return Err(sqlx::Error::RowNotFound);
        }
    };

    // Check if index already exists on the partition
    let index_exists = sqlx::query_scalar!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM pg_indexes
            WHERE schemaname = 'public'
              AND indexname = $1
              AND tablename = $2
        ) as "exists!"
        "#,
        index_name,
        partition_name
    )
    .fetch_one(pool)
    .await?;

    if !index_exists {
        // Create the index on the specific partition with correct dimensions
        let create_index_query = format!(
            r#"
            CREATE INDEX IF NOT EXISTS {}
            ON {} USING hnsw ((embedding::halfvec({})) halfvec_cosine_ops)
            "#,
            index_name, partition_name, dimensions
        );

        sqlx::query(&create_index_query)
            .execute(pool)
            .await?;

        tracing::info!("Created HNSW index {} on partition {} for instance {} with {} dimensions", 
                      index_name, partition_name, instance_id, dimensions);
    } else {
        tracing::debug!("Index {} already exists on partition {} for instance {}", index_name, partition_name, instance_id);
    }

    Ok(())
}

/// Complete reset of RAG instance state - drops partition and recreates fresh partition with index
pub async fn reset_rag_instance_state(instance_id: Uuid) -> Result<(), sqlx::Error> {
    tracing::info!("Starting complete state reset for RAG instance {}", instance_id);
    
    // Get the instance to check embedding model configuration
    let instance = get_rag_instance_by_id(instance_id).await?;
    let instance = match instance {
        Some(inst) => inst,
        None => {
            tracing::warn!("RAG instance {} not found during state reset", instance_id);
            return Ok(());
        }
    };
    
    // Clear processing pipeline
    clear_processing_pipeline(instance_id).await?;
    
    // Reset file processing status
    reset_processing_status(instance_id).await?;
    
    // Drop and recreate partition (this instantly clears all vector documents and indexes)
    if let Err(e) = drop_partition_for_instance(instance_id).await {
        tracing::warn!("Failed to drop partition for RAG instance {} during reset: {}", instance_id, e);
    }
    
    if let Err(e) = create_partition_for_instance(instance_id).await {
        tracing::warn!("Failed to create partition for RAG instance {} during reset: {}", instance_id, e);
        return Err(e);
    }
    
    // Create fresh index if embedding model is configured
    if instance.embedding_model_id.is_some() {
        if let Err(e) = create_rag_instance_index(instance_id).await {
            tracing::warn!("Failed to create index for RAG instance {} during reset: {}", instance_id, e);
        }
    }
    
    tracing::info!("Completed state reset for RAG instance {}", instance_id);
    
    Ok(())
}
