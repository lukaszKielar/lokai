use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Serialize, Deserialize, FromRow, Debug, Clone, PartialEq)]
pub struct Conversation {
    pub id: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Role {
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
}

impl core::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Assistant => write!(f, "assistant"),
            Role::System => write!(f, "system"),
            Role::User => write!(f, "user"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Message {
    pub id: u32,
    pub role: Role,
    pub content: String,
    pub conversation_id: u32,
    pub created_at: DateTime<Utc>,
}

impl FromRow<'_, SqliteRow> for Message {
    fn from_row(row: &'_ SqliteRow) -> sqlx::Result<Self> {
        let role = row
            .try_get("role")
            .map_err(|err| sqlx::Error::ColumnDecode {
                index: "role".to_string(),
                source: err.into(),
            });
        let role = match role {
            Ok("assistant") => Role::Assistant,
            Ok("system") => Role::System,
            Ok("user") => Role::User,
            Ok(other) => {
                return Err(sqlx::Error::ColumnDecode {
                    index: "role".to_string(),
                    source: format!("Expected one of [assistant, system, user], got [{}]", other)
                        .into(),
                })
            }
            Err(err) => return Err(err),
        };

        Ok(Message {
            id: row.try_get("id")?,
            role,
            content: row.try_get("content")?,
            conversation_id: row.try_get("conversation_id")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
