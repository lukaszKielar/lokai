use chrono::{DateTime, Utc};
use ratatui::text::Text;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, FromRow, Row};

#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct Conversation {
    pub id: i64,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "user")]
    User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: i64,
    pub role: Role,
    pub content: String,
    pub conversation_id: i64,
    pub created_at: DateTime<Utc>,
}

impl<'a> From<&Message> for Text<'a> {
    fn from(val: &Message) -> Self {
        let icon = match val.role {
            Role::Assistant => "🤖",
            Role::System => "⚙️",
            Role::User => "🤔",
        };
        format!("{} {}", icon, val.content).into()
    }
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
                    source: format!(
                        "Expected one of [assistant, system, user], got [{:?}]",
                        other
                    )
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
