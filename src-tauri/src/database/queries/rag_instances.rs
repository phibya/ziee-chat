use crate::database::{
    get_database_pool,
    models::{
        CreateRAGInstanceRequest, CreateSystemRAGInstanceRequest, RAGEngineSettings, RAGEngineType,
        RAGInstance, RAGInstanceListResponse, UpdateRAGInstanceRequest,
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
    let engine_settings_json = serde_json::to_value(&request.engine_settings.unwrap_or_default())
        .unwrap_or_else(|_| serde_json::json!({}));

    let instance = sqlx::query_as!(
        RAGInstance,
        r#"INSERT INTO rag_instances (
            id, provider_id, user_id, project_id, name, alias, description, 
            enabled, is_active, is_system, engine_type, 
            engine_settings, embedding_model_id, llm_model_id, parameters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, provider_id, user_id, project_id, name, alias, description, 
                  enabled, is_active, is_system, 
                  engine_type as "engine_type: RAGEngineType",
                  engine_settings as "engine_settings: RAGEngineSettings",
                  embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at"#,
        instance_id,
        request.provider_id,
        user_id,
        request.project_id,
        &request.name,
        &request.alias,
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
    let engine_settings_json = serde_json::to_value(&request.engine_settings.unwrap_or_default())
        .unwrap_or_else(|_| serde_json::json!({}));

    let instance = sqlx::query_as!(
        RAGInstance,
        r#"INSERT INTO rag_instances (
            id, provider_id, user_id, project_id, name, alias, description, 
            enabled, is_active, is_system, engine_type, 
            engine_settings, embedding_model_id, llm_model_id, parameters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, provider_id, user_id, project_id, name, alias, description, 
                  enabled, is_active, is_system, 
                  engine_type as "engine_type: RAGEngineType", 
                  engine_settings as "engine_settings: RAGEngineSettings",
                  embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at"#,
        instance_id,
        request.provider_id,
        None::<Uuid>, // user_id = null for system instances
        None::<Uuid>, // project_id = null for system instances
        &request.name,
        &request.alias,
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
            r#"SELECT id, provider_id, user_id, project_id, name, alias, description, 
                    enabled, is_active, is_system, 
                    engine_type as "engine_type: RAGEngineType", 
                    engine_settings as "engine_settings: RAGEngineSettings",
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
            r#"SELECT id, provider_id, user_id, project_id, name, alias, description, 
                    enabled, is_active, is_system, 
                    engine_type as "engine_type: RAGEngineType", 
                    engine_settings as "engine_settings: RAGEngineSettings",
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
            r#"SELECT DISTINCT ri.id, ri.provider_id, ri.user_id, ri.project_id, ri.name, ri.alias, ri.description, 
                    ri.enabled, ri.is_active, ri.is_system, 
                    ri.engine_type as "engine_type: RAGEngineType", 
                    ri.engine_settings as "engine_settings: RAGEngineSettings",
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
            r#"SELECT id, provider_id, user_id, project_id, name, alias, description, 
                    enabled, is_active, is_system, 
                    engine_type as "engine_type: RAGEngineType", 
                    engine_settings as "engine_settings: RAGEngineSettings",
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
        r#"SELECT id, provider_id, user_id, project_id, name, alias, description, 
                enabled, is_active, is_system, 
                engine_type as "engine_type: RAGEngineType", 
                engine_settings as "engine_settings: RAGEngineSettings",
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

    if let Some(engine_settings) = &engine_settings_update {
        sqlx::query!(
            "UPDATE rag_instances SET engine_settings = $1, updated_at = NOW() WHERE id = $2",
            engine_settings,
            instance_id
        )
        .execute(pool)
        .await?;
    }

    // Return the updated instance
    let instance = sqlx::query_as!(
        RAGInstance,
        r#"SELECT id, provider_id, user_id, project_id, name, alias, description, 
                  enabled, is_active, is_system, 
                  engine_type as "engine_type: RAGEngineType", 
                  engine_settings as "engine_settings: RAGEngineSettings",
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

    // Delete RAG instance (CASCADE will automatically delete associated files and rag_instance_files)
    let result = sqlx::query!("DELETE FROM rag_instances WHERE id = $1", instance_id)
        .execute(pool)
        .await?;

    let deleted = result.rows_affected() > 0;

    if deleted {
        // Clean up file system
        if let Err(e) = crate::global::RAG_FILE_STORAGE
            .delete_instance_files(instance_id)
            .await
        {
            eprintln!("Failed to delete RAG instance files from filesystem: {}", e);
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
