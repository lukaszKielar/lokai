use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, SqlitePool};

use crate::app::AppResult;

// TODO: move to models.rs
#[derive(Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

// TODO: move to models.rs
#[derive(Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: i64,
    // TODO: make role an enum
    pub role: String,
    pub content: String,
    pub conversation_id: i64,
    pub created_at: DateTime<Utc>,
}

pub async fn get_conversations(sqlite: SqlitePool) -> AppResult<Vec<Conversation>> {
    let items = sqlx::query_as("SELECT * FROM conversations ORDER BY created_at ASC")
        .fetch_all(&sqlite)
        .await?;

    Ok(items)
}

pub async fn get_conversation_messages(
    sqlite: SqlitePool,
    conversation_id: i64,
) -> AppResult<Vec<Message>> {
    let items =
        sqlx::query_as("SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC")
            .bind(conversation_id)
            .fetch_all(&sqlite)
            .await?;

    Ok(items)
}
