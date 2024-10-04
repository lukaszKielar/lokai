use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// TODO: include `session_path` in the model
#[derive(Serialize, Deserialize, FromRow, Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl Conversation {
    // TODO: implement load and save methods
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Role {
    Assistant,
    System,
    User,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub conversation_id: u32,
}

impl Message {
    fn new(role: Role, content: String, conversation_id: u32) -> Self {
        Self {
            role,
            content,
            conversation_id,
        }
    }

    pub fn user(content: String, conversation_id: u32) -> Self {
        Self::new(Role::User, content, conversation_id)
    }

    pub fn assistant(content: String, conversation_id: u32) -> Self {
        Self::new(Role::Assistant, content, conversation_id)
    }
}
