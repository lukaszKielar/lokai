use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Conversation {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Message {
    pub id: i64,
    // TODO: make role an enum
    pub role: String,
    pub content: String,
    pub conversation_id: i64,
    pub created_at: DateTime<Utc>,
}
