use sqlx::SqlitePool;
use tracing::debug;
use uuid::Uuid;

use crate::error::Result;
use crate::models::{Conversation, ConversationSettings, Message};

pub async fn get_conversation_messages(
    sqlite: SqlitePool,
    conversation_id: Uuid,
) -> Result<Vec<Message>> {
    let messages: Vec<Message> = sqlx::query_as(
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

    Ok(messages)
}

// TODO: automatically generate id, I shouldn't create it on a client side
pub async fn create_message(sqlite: SqlitePool, message: Message) -> Result<Message> {
    debug!(message_id = message.id.to_string(), "saving message to db");

    let new_message: Message = sqlx::query_as(
        r#"
INSERT INTO messages ( id, role, content, conversation_id, created_at )
VALUES ( ?1, ?2, ?3, ?4, ?5 )
RETURNING *
        "#,
    )
    .bind(message.id)
    .bind(message.role)
    .bind(message.content)
    .bind(message.conversation_id)
    .bind(message.created_at)
    .fetch_one(&sqlite)
    .await?;

    debug!(
        message_id = new_message.id.to_string(),
        "message saved to db"
    );

    Ok(new_message)
}

pub async fn get_conversation(
    sqlite: SqlitePool,
    conversation_id: Uuid,
) -> Result<Option<Conversation>> {
    debug!(
        conversation_id = conversation_id.to_string(),
        "getting conversation from db"
    );
    let maybe_conversation: Option<Conversation> = sqlx::query_as(
        r#"
SELECT *
FROM conversations
WHERE id = ?
        "#,
    )
    .bind(conversation_id)
    .fetch_optional(&sqlite)
    .await?;

    Ok(maybe_conversation)
}

pub async fn get_conversations(sqlite: SqlitePool) -> Result<Vec<Conversation>> {
    let conversations: Vec<Conversation> = sqlx::query_as(
        r#"
SELECT *
FROM conversations
ORDER BY created_at ASC
        "#,
    )
    .fetch_all(&sqlite)
    .await?;

    Ok(conversations)
}

pub async fn create_conversation(
    sqlite: SqlitePool,
    conversation: Conversation,
    llm_model: String,
) -> Result<Conversation> {
    debug!(
        conversation_id = conversation.id.to_string(),
        "saving conversation to db"
    );

    let mut transaction = sqlite.begin().await?;

    let settings = ConversationSettings::new(llm_model, conversation.id);
    let new_conversation: Conversation = sqlx::query_as(
        r#"
INSERT INTO conversations ( id, name, created_at )
VALUES ( ?1, ?2, ?3 )
RETURNING *
        "#,
    )
    .bind(conversation.id)
    .bind(conversation.name)
    .bind(conversation.created_at)
    .fetch_one(&mut *transaction)
    .await?;

    let _: ConversationSettings = sqlx::query_as(
        r#"
INSERT INTO conversation_settings ( id, llm_model, conversation_id, created_at )
VALUES ( ?1, ?2, ?3, ?4 )
RETURNING *
        "#,
    )
    .bind(settings.id)
    .bind(settings.llm_model)
    .bind(settings.conversation_id)
    .bind(settings.created_at)
    .fetch_one(&mut *transaction)
    .await?;

    debug!(
        conversation_id = new_conversation.id.to_string(),
        "conversation saved to db"
    );

    transaction.commit().await?;

    Ok(new_conversation)
}

pub async fn create_conversation_if_not_exists(
    sqlite: SqlitePool,
    conversation: Conversation,
    llm_model: String,
) -> Result<Conversation> {
    if let Some(conversation) = get_conversation(sqlite.clone(), conversation.id).await? {
        debug!(
            conversation_id = conversation.id.to_string(),
            "conversation already exist in db"
        );
        return Ok(conversation);
    } else {
        return create_conversation(sqlite, conversation, llm_model).await;
    }
}

pub async fn delete_conversation(
    sqlite: SqlitePool,
    conversation_id: Uuid,
) -> Result<Option<Conversation>> {
    debug!(
        conversation_id = conversation_id.to_string(),
        "deleting conversation from db"
    );

    let maybe_conversation: Option<Conversation> = sqlx::query_as(
        r#"
        DELETE FROM conversations
        WHERE id = ?1
        RETURNING *
        "#,
    )
    .bind(conversation_id)
    .fetch_optional(&sqlite)
    .await?;

    match maybe_conversation {
        Some(conversation) => {
            debug!(
                conversation_id = conversation_id.to_string(),
                "conversation deleted from db"
            );
            Ok(Some(conversation))
        }
        None => {
            debug!(
                conversation_id = conversation_id.to_string(),
                "conversation not found"
            );
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::models::Role;

    use super::*;
    use sqlx::Row;

    static LLM_MODEL: &'static str = "test-model";

    async fn table_count(sqlite: SqlitePool, table_name: &str) -> Result<i64> {
        let query = format!("SELECT COUNT(*) FROM {table_name}");
        let count = sqlx::query(&query)
            .bind(table_name)
            .fetch_one(&sqlite)
            .await?;

        Ok(count.get(0))
    }

    #[sqlx::test]
    async fn test_create_conversation_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 0);
        assert_eq!(table_count(pool.clone(), "conversation_settings").await?, 0);

        // when:
        let new_conversation = create_conversation(
            pool.clone(),
            Conversation::new("name".to_string()),
            LLM_MODEL.to_string(),
        )
        .await?;

        // then:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 1);
        assert_eq!(table_count(pool, "conversation_settings").await?, 1);
        assert_eq!(new_conversation.name, "name");

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_message_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        let conversation = create_conversation(
            pool.clone(),
            Conversation::new("name".to_string()),
            LLM_MODEL.to_string(),
        )
        .await?;
        assert_eq!(table_count(pool.clone(), "conversations").await?, 1);
        assert_eq!(table_count(pool.clone(), "conversation_settings").await?, 1);
        assert_eq!(table_count(pool.clone(), "messages").await?, 0);

        // when:
        let new_message = create_message(
            pool.clone(),
            Message::user("content".to_string(), conversation.id),
        )
        .await?;

        // then:
        assert_eq!(table_count(pool, "messages").await?, 1);
        assert_eq!(new_message.role, Role::User.to_string());
        assert_eq!(new_message.content, "content");
        assert_eq!(new_message.conversation_id, conversation.id);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_conversation_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        let conversation = create_conversation(
            pool.clone(),
            Conversation::new("name".to_string()),
            LLM_MODEL.to_string(),
        )
        .await?;

        // when:
        let maybe_conversation = get_conversation(pool, conversation.id).await?;

        // then:
        assert!(maybe_conversation.is_some());
        assert_eq!(maybe_conversation.unwrap(), conversation);

        Ok(())
    }

    #[sqlx::test]
    async fn test_get_conversation_messages_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:

        // when:
        let maybe_messages = get_conversation_messages(pool, Uuid::new_v4()).await?;

        // then:
        assert_eq!(maybe_messages, Vec::new());

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_conversation_if_not_exists_row_already_exists_ok(
        pool: sqlx::SqlitePool,
    ) -> Result<()> {
        // given:
        let conversation = Conversation::new("test".to_string());
        let _ =
            create_conversation(pool.clone(), conversation.clone(), LLM_MODEL.to_string()).await?;
        assert_eq!(table_count(pool.clone(), "conversations").await?, 1);
        assert_eq!(table_count(pool.clone(), "conversation_settings").await?, 1);

        // when:
        let already_existing_conversation = create_conversation_if_not_exists(
            pool.clone(),
            conversation.clone(),
            LLM_MODEL.to_string(),
        )
        .await?;

        // then:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 1);
        assert_eq!(table_count(pool, "conversation_settings").await?, 1);
        assert_eq!(conversation, already_existing_conversation);

        Ok(())
    }

    #[sqlx::test]
    async fn test_create_conversation_if_not_exists_row_doesnt_exists_ok(
        pool: sqlx::SqlitePool,
    ) -> Result<()> {
        // given:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 0);
        assert_eq!(table_count(pool.clone(), "conversation_settings").await?, 0);

        // when:
        let _ = create_conversation_if_not_exists(
            pool.clone(),
            Conversation::new("test".to_string()),
            LLM_MODEL.to_string(),
        )
        .await?;

        // then:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 1);
        assert_eq!(table_count(pool, "conversation_settings").await?, 1);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_conversation_which_exist_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        let conversation = Conversation::new("test".to_string());
        let _ =
            create_conversation(pool.clone(), conversation.clone(), LLM_MODEL.to_string()).await?;
        assert_eq!(table_count(pool.clone(), "conversations").await?, 1);
        assert_eq!(table_count(pool.clone(), "conversation_settings").await?, 1);

        // when:
        let maybe_deleted_conversation =
            delete_conversation(pool.clone(), conversation.id.clone()).await?;

        // then:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 0);
        assert_eq!(table_count(pool, "conversation_settings").await?, 1);
        assert!(maybe_deleted_conversation.is_some());
        assert_eq!(conversation, maybe_deleted_conversation.unwrap());

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete_conversation_which_doesnt_exist_ok(pool: sqlx::SqlitePool) -> Result<()> {
        // given:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 0);
        assert_eq!(table_count(pool.clone(), "conversation_settings").await?, 0);

        // when:
        let maybe_deleted_conversation = delete_conversation(pool.clone(), Uuid::new_v4()).await?;

        // then:
        assert_eq!(table_count(pool.clone(), "conversations").await?, 0);
        assert_eq!(table_count(pool, "conversation_settings").await?, 0);
        assert!(maybe_deleted_conversation.is_none());

        Ok(())
    }
}
