use crate::database::models::Branch;
use sqlx::Error;
use uuid::Uuid;

/// Create a new branch within a transaction
pub async fn create_branch_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    conversation_id: Uuid,
    _name: Option<String>,
) -> Result<Branch, Error> {
    let branch_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    // Insert the branch
    sqlx::query(
        r#"
        INSERT INTO branches (id, conversation_id, created_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(branch_id)
    .bind(conversation_id)
    .bind(now)
    .execute(&mut **tx)
    .await?;

    Ok(Branch {
        id: branch_id,
        conversation_id,
        created_at: now,
    })
}
