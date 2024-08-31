use sqlx::SqlitePool;

use crate::app::AppResult;
use crate::models::{Conversation, Message, Role};

pub async fn get_conversations(sqlite: SqlitePool) -> AppResult<Vec<Conversation>> {
    let items = sqlx::query_as(
        r#"
        SELECT *
        FROM conversations
        ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&sqlite)
    .await?;

    Ok(items)
}

pub async fn get_messages(sqlite: SqlitePool, conversation_id: u32) -> AppResult<Vec<Message>> {
    let items = sqlx::query_as(
        r#"
            SELECT *
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at ASC
            "#,
    )
    .bind(conversation_id)
    .fetch_all(&sqlite)
    .await?;

    Ok(items)
}

pub async fn create_message(
    sqlite: SqlitePool,
    role: Role,
    content: String,
    conversation_id: u32,
) -> AppResult<Message> {
    let new_message: Message = sqlx::query_as(
        r#"
        INSERT INTO messages(role, content, conversation_id)
        VALUES (?1, ?2, ?3)
        RETURNING *
    "#,
    )
    .bind(role.to_string())
    .bind(content)
    .bind(conversation_id)
    .fetch_one(&sqlite)
    .await?;

    Ok(new_message)
}
