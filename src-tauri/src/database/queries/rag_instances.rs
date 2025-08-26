use uuid::Uuid;
use crate::database::{
    get_database_pool,
    models::{
        CreateRAGInstanceRequest, CreateSystemRAGInstanceRequest, UpdateRAGInstanceRequest, 
        RAGInstance, RAGInstanceListResponse
    },
    queries::user_group_rag_providers::can_user_create_rag_instance,
};

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
        eprintln!("User {} cannot create instances with provider {}", user_id, request.provider_id);
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

/// Get RAG instance by ID with ownership validation
pub async fn get_rag_instance(instance_id: Uuid) -> Result<Option<RAGInstance>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let instance: Option<RAGInstance> = sqlx::query_as(
        "SELECT id, provider_id, user_id, project_id, name, alias, description, 
                enabled, is_active, is_system, engine_type, 
                engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances 
         WHERE id = $1",
    )
    .bind(instance_id)
    .fetch_optional(pool)
    .await?;

    Ok(instance)
}

/// List user's RAG instances (is_system = false, owned by user)
pub async fn list_user_rag_instances(
    user_id: Uuid,
    page: i32,
    per_page: i32,
) -> Result<RAGInstanceListResponse, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    // Get total count of user's instances
    let total_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM rag_instances WHERE user_id = $1 AND is_system = false"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    // Get user's instances with pagination
    let instances: Vec<RAGInstance> = sqlx::query_as(
        "SELECT id, provider_id, user_id, project_id, name, alias, description, 
                enabled, is_active, is_system, engine_type, 
                engine_settings, embedding_model_id, llm_model_id, age_graph_name, parameters, 
                created_at, updated_at
         FROM rag_instances 
         WHERE user_id = $1 AND is_system = false
         ORDER BY created_at DESC 
         LIMIT $2 OFFSET $3",
    )
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
    let total_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM rag_instances WHERE is_system = true"
    )
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

/// Delete RAG instance with ownership validation
pub async fn delete_rag_instance(instance_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query("DELETE FROM rag_instances WHERE id = $1")
        .bind(instance_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

/// Validate user owns RAG instance or it's a system instance
pub async fn validate_rag_instance_access(
    user_id: Uuid,
    instance_id: Uuid,
    require_owner: bool,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let has_access: Option<(bool,)> = if require_owner {
        // User must own the instance
        sqlx::query_as(
            "SELECT true FROM rag_instances 
             WHERE id = $1 AND user_id = $2 AND is_system = false
             LIMIT 1",
        )
        .bind(instance_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
    } else {
        // User can access if they own it or if it's a system instance
        sqlx::query_as(
            "SELECT true FROM rag_instances 
             WHERE id = $1 AND (user_id = $2 OR is_system = true)
             LIMIT 1",
        )
        .bind(instance_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?
    };

    Ok(has_access.is_some())
}