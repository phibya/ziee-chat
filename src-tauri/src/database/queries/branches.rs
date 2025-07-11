use super::get_database_pool;
use crate::database::models::Branch;
use sqlx::Error;
use uuid::Uuid;

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
        "#,
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

/// Update conversation's active branch
pub async fn set_active_branch(conversation_id: Uuid, branch_id: Uuid) -> Result<bool, Error> {
    let pool = get_database_pool()?;
    let pool = pool.as_ref();

    let result = sqlx::query("UPDATE conversations SET active_branch_id = $1 WHERE id = $2")
        .bind(branch_id)
        .bind(conversation_id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
