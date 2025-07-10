use super::get_database_pool;
use crate::database::models::{Branch, BranchDb};
use sqlx::Error;
use uuid::Uuid;

/// Create a new branch for a conversation
pub async fn create_branch(
    conversation_id: Uuid,
    name: Option<String>,
) -> Result<Branch, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    let branch_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Insert the branch
    sqlx::query(
        r#"
        INSERT INTO branches (id, conversation_id, name, created_at)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(branch_id)
    .bind(conversation_id)
    .bind(&name)
    .bind(now)
    .execute(pool)
    .await?;
    
    Ok(Branch {
        id: branch_id,
        conversation_id,
        name,
        created_at: now,
    })
}

/// Create a new branch within a transaction
pub async fn create_branch_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    conversation_id: Uuid,
    name: Option<String>,
) -> Result<Branch, Error> {
    let branch_id = Uuid::new_v4();
    let now = chrono::Utc::now();
    
    // Insert the branch
    sqlx::query(
        r#"
        INSERT INTO branches (id, conversation_id, name, created_at)
        VALUES ($1, $2, $3, $4)
        "#
    )
    .bind(branch_id)
    .bind(conversation_id)
    .bind(&name)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    
    Ok(Branch {
        id: branch_id,
        conversation_id,
        name,
        created_at: now,
    })
}

/// Get a branch by ID
pub async fn get_branch_by_id(branch_id: Uuid) -> Result<Option<Branch>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    let row = sqlx::query_as::<_, BranchDb>(
        "SELECT id, conversation_id, name, created_at FROM branches WHERE id = $1"
    )
    .bind(branch_id)
    .fetch_optional(pool)
    .await?;
    
    match row {
        Some(branch_db) => Ok(Some(Branch {
            id: branch_db.id,
            conversation_id: branch_db.conversation_id,
            name: branch_db.name,
            created_at: branch_db.created_at,
        })),
        None => Ok(None),
    }
}

/// Get all branches for a conversation
pub async fn get_conversation_branches(conversation_id: Uuid) -> Result<Vec<Branch>, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    let rows = sqlx::query_as::<_, BranchDb>(
        "SELECT id, conversation_id, name, created_at FROM branches WHERE conversation_id = $1 ORDER BY created_at ASC"
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;
    
    let branches = rows
        .into_iter()
        .map(|branch_db| Branch {
            id: branch_db.id,
            conversation_id: branch_db.conversation_id,
            name: branch_db.name,
            created_at: branch_db.created_at,
        })
        .collect();
    
    Ok(branches)
}

/// Delete a branch and all its messages
pub async fn delete_branch(branch_id: Uuid) -> Result<bool, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    // Start transaction
    let mut tx = pool.begin().await?;
    
    // Delete all messages in this branch
    sqlx::query("DELETE FROM messages WHERE new_branch_id = $1")
        .bind(branch_id)
        .execute(&mut *tx)
        .await?;
    
    // Delete the branch
    let result = sqlx::query("DELETE FROM branches WHERE id = $1")
        .bind(branch_id)
        .execute(&mut *tx)
        .await?;
    
    // Commit transaction
    tx.commit().await?;
    
    Ok(result.rows_affected() > 0)
}

/// Update conversation's active branch
pub async fn set_active_branch(
    conversation_id: Uuid,
    branch_id: Uuid,
) -> Result<bool, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();
    
    let result = sqlx::query(
        "UPDATE conversations SET active_branch_id = $1 WHERE id = $2"
    )
    .bind(branch_id)
    .bind(conversation_id)
    .execute(pool)
    .await?;
    
    Ok(result.rows_affected() > 0)
}