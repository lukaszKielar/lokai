use std::time::Duration;

use sqlx::SqlitePool;

use crate::{
    models::{Conversation, Message, Role},
    AppResult,
};

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
    tokio::time::sleep(Duration::from_millis(10)).await;
    let items = sqlx::query_as(
        r#"
            SELECT *
            FROM messages
            WHERE conversation_id = ?1
            ORDER BY created_at ASC
            "#,
    )
    .bind(conversation_id)
    .fetch_all(&sqlite)
    .await?;

    Ok(items)
}

// TODO: fix transactions
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

// TODO: fix transactions
pub async fn update_message(
    sqlite: SqlitePool,
    content: String,
    message_id: u32,
) -> AppResult<Message> {
    let updated_message: Message = sqlx::query_as(
        r#"
        UPDATE messages
        SET content = ?1
        WHERE id = ?2
        RETURNING *
        "#,
    )
    .bind(content)
    .bind(message_id)
    .fetch_one(&sqlite)
    .await?;

    Ok(updated_message)
}
