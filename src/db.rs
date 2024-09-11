use std::time::Duration;

use sqlx::{Executor, Sqlite};

use crate::{
    models::{Conversation, Message, Role},
    AppResult,
};

pub async fn get_conversations<'e, E>(executor: E) -> AppResult<Vec<Conversation>>
where
    E: Executor<'e, Database = Sqlite>,
{
    let items = sqlx::query_as(
        r#"
        SELECT *
        FROM conversations
        ORDER BY created_at ASC
        "#,
    )
    .persistent(false)
    .fetch_all(executor)
    .await?;

    Ok(items)
}

pub async fn get_messages<'e, E>(executor: E, conversation_id: u32) -> AppResult<Vec<Message>>
where
    E: Executor<'e, Database = Sqlite>,
{
    // TODO: for some weird reason query is cached despite setting persistence to false
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
    .persistent(false)
    .fetch_all(executor)
    .await?;

    Ok(items)
}

pub async fn create_message<'e, E>(
    executor: E,
    role: Role,
    content: String,
    conversation_id: u32,
) -> AppResult<Message>
where
    E: Executor<'e, Database = Sqlite>,
{
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
    .persistent(false)
    .fetch_one(executor)
    .await?;

    Ok(new_message)
}

pub async fn update_message<'e, E>(
    executor: E,
    content: String,
    message_id: u32,
) -> AppResult<Message>
where
    E: Executor<'e, Database = Sqlite>,
{
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
    .persistent(false)
    .fetch_one(executor)
    .await?;

    Ok(updated_message)
}
