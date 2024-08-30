use sqlx::SqlitePool;

use crate::app::AppResult;
use crate::models::{Conversation, Message};

pub async fn get_conversations(sqlite: SqlitePool) -> AppResult<Vec<Conversation>> {
    let items = sqlx::query_as("SELECT * FROM conversations ORDER BY created_at ASC")
        .fetch_all(&sqlite)
        .await?;

    Ok(items)
}

pub async fn get_messages(sqlite: SqlitePool, conversation_id: i64) -> AppResult<Vec<Message>> {
    let items =
        sqlx::query_as("SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC")
            .bind(conversation_id)
            .fetch_all(&sqlite)
            .await?;

    Ok(items)
}
