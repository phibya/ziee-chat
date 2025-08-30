use crate::database::{
    get_database_pool,
    models::{
        CreateRAGInstanceRequest, CreateSystemRAGInstanceRequest, RAGInstance,
        RAGInstanceListResponse, UpdateRAGInstanceRequest,
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

    let instance: RAGInstance = sqlx::query_as(
        "INSERT INTO rag_instances (
            id, provider_id, user_id, project_id, name, alias, description, 
            enabled, is_active, is_system, engine_type, 
            engine_settings, embedding_model_id, llm_model_id, parameters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, provider_id, user_id, project_id, name, alias, description, 
                  enabled, is_active, is_system, engine_type, 
                  engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at",
    )
    .bind(instance_id)
    .bind(request.provider_id)
    .bind(user_id)
    .bind(request.project_id)
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(true) // enabled = true by default
    .bind(false) // is_active = false by default
    .bind(false) // is_system = false for user instances
    .bind(engine_type_str)
    .bind(&engine_settings_json)
    .bind(request.embedding_model_id)
    .bind(request.llm_model_id)
    .bind(request.parameters.unwrap_or_else(|| serde_json::json!({})))
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

    let instance: RAGInstance = sqlx::query_as(
        "INSERT INTO rag_instances (
            id, provider_id, user_id, project_id, name, alias, description, 
            enabled, is_active, is_system, engine_type, 
            engine_settings, embedding_model_id, llm_model_id, parameters
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, provider_id, user_id, project_id, name, alias, description, 
                  enabled, is_active, is_system, engine_type, 
                  engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                  created_at, updated_at",
    )
    .bind(instance_id)
    .bind(request.provider_id)
    .bind(None::<Uuid>) // user_id = null for system instances
    .bind(None::<Uuid>) // project_id = null for system instances
    .bind(&request.name)
    .bind(&request.alias)
    .bind(&request.description)
    .bind(true) // enabled = true by default
    .bind(false) // is_active = false by default
    .bind(true) // is_system = true for system instances
    .bind(engine_type_str)
    .bind(&engine_settings_json)
    .bind(request.embedding_model_id)
    .bind(request.llm_model_id)
    .bind(request.parameters.unwrap_or_else(|| serde_json::json!({})))
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

    let query = if has_admin_read {
        // Admin users can see all instances (enabled and disabled)
        "SELECT id, provider_id, user_id, project_id, name, alias, description, 
                enabled, is_active, is_system, engine_type, 
                engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances 
         WHERE id = $1"
    } else {
        // Regular users can only see enabled instances
        "SELECT id, provider_id, user_id, project_id, name, alias, description, 
                enabled, is_active, is_system, engine_type, 
                engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances 
         WHERE id = $1 AND enabled = true"
    };

    let instance: Option<RAGInstance> = sqlx::query_as(query)
        .bind(instance_id)
        .fetch_optional(pool)
        .await?;

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

    let (count_query, list_query) = if include_sys {
        // Include both user instances AND accessible system instances (only enabled ones)
        (
            "SELECT COUNT(DISTINCT ri.id) FROM rag_instances ri
             LEFT JOIN user_group_rag_providers ugrp ON ri.provider_id = ugrp.provider_id AND ri.is_system = true
             LEFT JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id AND ugm.user_id = $1
             LEFT JOIN user_groups ug ON ugm.group_id = ug.id AND ug.is_active = true
             WHERE ((ri.user_id = $1 AND ri.is_system = false AND ri.enabled = true) 
             OR (ri.is_system = true AND ri.enabled = true AND ugm.user_id IS NOT NULL))",
             
            "SELECT DISTINCT ri.id, ri.provider_id, ri.user_id, ri.project_id, ri.name, ri.alias, ri.description, 
                    ri.enabled, ri.is_active, ri.is_system, ri.engine_type, 
                    ri.engine_settings, ri.embedding_model_id, ri.llm_model_id, ri.age_graph_name, ri.parameters, 
                    ri.created_at, ri.updated_at
             FROM rag_instances ri
             LEFT JOIN user_group_rag_providers ugrp ON ri.provider_id = ugrp.provider_id AND ri.is_system = true
             LEFT JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id AND ugm.user_id = $1
             LEFT JOIN user_groups ug ON ugm.group_id = ug.id AND ug.is_active = true
             WHERE ((ri.user_id = $1 AND ri.is_system = false AND ri.enabled = true) 
             OR (ri.is_system = true AND ri.enabled = true AND ugm.user_id IS NOT NULL))
             ORDER BY ri.is_system ASC, ri.created_at DESC 
             LIMIT $2 OFFSET $3"
        )
    } else {
        // Only user instances (only enabled ones)
        (
            "SELECT COUNT(*) FROM rag_instances WHERE user_id = $1 AND is_system = false AND enabled = true",
            "SELECT id, provider_id, user_id, project_id, name, alias, description, 
                    enabled, is_active, is_system, engine_type, 
                    engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                    created_at, updated_at
             FROM rag_instances 
             WHERE user_id = $1 AND is_system = false AND enabled = true
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3"
        )
    };

    // Get total count
    let total_count: (i64,) = sqlx::query_as(count_query)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    // Get instances with pagination
    let instances: Vec<RAGInstance> = sqlx::query_as(list_query)
        .bind(user_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok(RAGInstanceListResponse {
        instances,
        total: total_count.0,
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
    let total_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM rag_instances WHERE is_system = true")
            .fetch_one(pool)
            .await?;

    // Get system instances with pagination
    let instances: Vec<RAGInstance> = sqlx::query_as(
        "SELECT id, provider_id, user_id, project_id, name, alias, description, 
                enabled, is_active, is_system, engine_type, 
                engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances 
         WHERE is_system = true
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2",
    )
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(RAGInstanceListResponse {
        instances,
        total: total_count.0,
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

    let instance: Option<RAGInstance> = sqlx::query_as(
        "UPDATE rag_instances 
         SET name = COALESCE($2, name),
             description = COALESCE($3, description),
             enabled = COALESCE($4, enabled),
             embedding_model_id = COALESCE($5, embedding_model_id),
             llm_model_id = COALESCE($6, llm_model_id),
             parameters = COALESCE($7, parameters),
             engine_settings = COALESCE($8, engine_settings)
         WHERE id = $1
         RETURNING id, provider_id, user_id, project_id, name, alias, description, 
                   enabled, is_active, is_system, engine_type, 
                   engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                   created_at, updated_at",
    )
    .bind(instance_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(&request.enabled)
    .bind(&request.embedding_model_id)
    .bind(&request.llm_model_id)
    .bind(&request.parameters)
    .bind(&engine_settings_update)
    .fetch_optional(pool)
    .await?;

    Ok(instance)
}

/// Delete RAG instance with automatic CASCADE cleanup
pub async fn delete_rag_instance(instance_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Delete RAG instance (CASCADE will automatically delete associated files and rag_instance_files)
    let result = sqlx::query("DELETE FROM rag_instances WHERE id = $1")
        .bind(instance_id)
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
    let query_result: Option<(Uuid, Option<Uuid>, bool, Uuid)> = sqlx::query_as(
        "SELECT ri.id, ri.user_id, ri.is_system, ri.provider_id
         FROM rag_instances ri 
         WHERE ri.id = $1",
    )
    .bind(instance_id)
    .fetch_optional(pool)
    .await?;

    let (_, instance_user_id, is_system, provider_id) = match query_result {
        Some(result) => result,
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
    let has_provider_access: Option<(bool,)> = sqlx::query_as(
        "SELECT true FROM user_group_rag_providers ugrp
         JOIN user_group_memberships ugm ON ugrp.group_id = ugm.group_id
         JOIN user_groups ug ON ugm.group_id = ug.id
         WHERE ugm.user_id = $1 AND ug.is_active = true 
         AND ugrp.provider_id = $2
         LIMIT 1",
    )
    .bind(user_id)
    .bind(provider_id)
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
