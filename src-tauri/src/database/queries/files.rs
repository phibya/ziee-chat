use crate::database::{get_database_pool, models::file::*};
use sqlx::Row;
use uuid::Uuid;

pub async fn create_file(data: FileCreateData) -> Result<File, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let file = sqlx::query_as::<_, File>(
        r#"
        INSERT INTO files (
            id, user_id, filename, file_size, mime_type, 
            checksum, project_id, rag_instance_id, thumbnail_count, page_count, processing_metadata
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING *
        "#,
    )
    .bind(data.id)
    .bind(data.user_id)
    .bind(data.filename)
    .bind(data.file_size)
    .bind(data.mime_type)
    .bind(data.checksum)
    .bind(data.project_id)
    .bind(data.rag_instance_id)
    .bind(data.thumbnail_count)
    .bind(data.page_count)
    .bind(data.processing_metadata)
    .fetch_one(pool)
    .await?;

    Ok(file)
}

pub async fn get_file_by_id(file_id: Uuid) -> Result<Option<File>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = $1")
        .bind(file_id)
        .fetch_optional(pool)
        .await?;

    Ok(file)
}

pub async fn get_file_by_id_and_user(
    file_id: Uuid,
    user_id: Uuid,
) -> Result<Option<File>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let file = sqlx::query_as::<_, File>("SELECT * FROM files WHERE id = $1 AND user_id = $2")
        .bind(file_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    Ok(file)
}

pub async fn get_files_by_project(
    project_id: Uuid,
    user_id: Uuid,
    page: i32,
    per_page: i32,
) -> Result<(Vec<File>, i64), sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let offset = (page - 1) * per_page;

    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT * FROM files 
        WHERE project_id = $1 AND user_id = $2
        ORDER BY created_at DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .bind(per_page as i64)
    .bind(offset as i64)
    .fetch_all(pool)
    .await?;

    let total_row =
        sqlx::query("SELECT COUNT(*) as count FROM files WHERE project_id = $1 AND user_id = $2")
            .bind(project_id)
            .bind(user_id)
            .fetch_one(pool)
            .await?;

    let total: i64 = total_row.get("count");

    Ok((files, total))
}

pub async fn get_files_by_message(
    message_id: Uuid,
    user_id: Uuid,
) -> Result<Vec<File>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let files = sqlx::query_as::<_, File>(
        r#"
        SELECT f.* FROM files f
        INNER JOIN messages_files mf ON f.id = mf.file_id
        WHERE mf.message_id = $1 AND f.user_id = $2
        ORDER BY mf.created_at ASC
        "#,
    )
    .bind(message_id)
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(files)
}

pub async fn delete_file(file_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    // Check if file has message associations
    if check_file_has_message_associations(file_id).await? {
        return Err(sqlx::Error::Protocol(
            "Cannot delete file that is associated with messages".into(),
        ));
    }

    let result = sqlx::query("DELETE FROM files WHERE id = $1 AND user_id = $2")
        .bind(file_id)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

// Message-file relationship functions

pub async fn delete_message_file_relationship(
    message_id: Uuid,
    file_id: Uuid,
) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query("DELETE FROM messages_files WHERE message_id = $1 AND file_id = $2")
        .bind(message_id)
        .bind(file_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn check_file_has_message_associations(file_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let row = sqlx::query("SELECT COUNT(*) as count FROM messages_files WHERE file_id = $1")
        .bind(file_id)
        .fetch_one(pool)
        .await?;

    let count: i64 = row.get("count");
    Ok(count > 0)
}

pub async fn check_file_has_project_association(file_id: Uuid) -> Result<bool, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let row =
        sqlx::query("SELECT COUNT(*) as count FROM files WHERE id = $1 AND project_id IS NOT NULL")
            .bind(file_id)
            .fetch_one(pool)
            .await?;

    let count: i64 = row.get("count");
    Ok(count > 0)
}

// Provider-file relationship functions
pub async fn create_provider_file_mapping(
    file_id: Uuid,
    provider_id: Uuid,
    provider_file_id: Option<String>,
    provider_metadata: serde_json::Value,
) -> Result<ProviderFile, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_file = sqlx::query_as::<_, ProviderFile>(
        r#"
        INSERT INTO provider_files (file_id, provider_id, provider_file_id, provider_metadata)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (file_id, provider_id) DO UPDATE SET
            provider_file_id = EXCLUDED.provider_file_id,
            provider_metadata = EXCLUDED.provider_metadata
        RETURNING *
        "#,
    )
    .bind(file_id)
    .bind(provider_id)
    .bind(provider_file_id)
    .bind(provider_metadata)
    .fetch_one(pool)
    .await?;

    Ok(provider_file)
}

pub async fn get_provider_file_mapping(
    file_id: Uuid,
    provider_id: Uuid,
) -> Result<Option<ProviderFile>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_file = sqlx::query_as::<_, ProviderFile>(
        "SELECT * FROM provider_files WHERE file_id = $1 AND provider_id = $2",
    )
    .bind(file_id)
    .bind(provider_id)
    .fetch_optional(pool)
    .await?;

    Ok(provider_file)
}

pub async fn get_provider_file_mappings_by_file(
    file_id: Uuid,
) -> Result<Vec<ProviderFile>, sqlx::Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let provider_files =
        sqlx::query_as::<_, ProviderFile>("SELECT * FROM provider_files WHERE file_id = $1")
            .bind(file_id)
            .fetch_all(pool)
            .await?;

    Ok(provider_files)
}
