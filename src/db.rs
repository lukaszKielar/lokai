use sqlx::{Executor, Sqlite};

use crate::{
    models::{Conversation, Message, Role},
    AppResult,
};

pub async fn get_conversation<'e, E>(executor: E, conversation_id: u32) -> AppResult<Conversation>
where
    E: Executor<'e, Database = Sqlite>,
{
    let conversation = sqlx::query_as(
        r#"
        SELECT *
        FROM conversations
        WHERE id = ?1
        "#,
    )
    .bind(conversation_id)
    .persistent(false)
    .fetch_one(executor)
    .await?;

    Ok(conversation)
}

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

pub async fn create_conversation<'e, E>(
    executor: E,
    name: &str,
    session_path: &str,
) -> AppResult<Conversation>
where
    E: Executor<'e, Database = Sqlite>,
{
    let conversation = sqlx::query_as(
        r#"
        INSERT INTO conversations(name, session_path) VALUES (?1, ?2)
        RETURNING *
        "#,
    )
    .bind(name)
    .bind(session_path)
    .persistent(false)
    .fetch_one(executor)
    .await?;

    Ok(conversation)
}

pub async fn delete_conversation<'e, E>(
    executor: E,
    conversation_id: u32,
) -> AppResult<Conversation>
where
    E: Executor<'e, Database = Sqlite>,
{
    let conversation = sqlx::query_as(
        r#"
        DELETE FROM conversations
        WHERE id = ?1
        RETURNING *
        "#,
    )
    .bind(conversation_id)
    .persistent(false)
    .fetch_one(executor)
    .await?;

    Ok(conversation)
}

pub async fn get_messages<'e, E>(executor: E, conversation_id: u32) -> AppResult<Vec<Message>>
where
    E: Executor<'e, Database = Sqlite>,
{
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
    content: &str,
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
    content: &str,
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

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use sqlx::{Row, SqlitePool};

    use crate::models::Role;

    use super::*;

    async fn table_count<'e, E>(executor: E, table_name: &str) -> AppResult<i64>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        let query = format!("SELECT COUNT(*) FROM {table_name}");
        let count = sqlx::query(&query)
            .persistent(false)
            .fetch_one(executor)
            .await?;

        Ok(count.get(0))
    }

    #[sqlx::test]
    async fn test_get_conversations_empty_table(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 0);

        // when
        let conversations = get_conversations(&pool).await?;

        // then
        assert_eq!(conversations, vec![]);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql"))]
    async fn test_get_conversations_non_empty_table(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);

        // when
        let conversations = get_conversations(&pool).await?;

        // then
        assert_eq!(
            conversations,
            vec![
                Conversation {
                    id: 1,
                    name: "conversation 1".to_string(),
                    session_path: "~/.lokai/chats/1".to_string(),
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:00Z")
                        .unwrap()
                        .into()
                },
                Conversation {
                    id: 2,
                    name: "conversation 2".to_string(),
                    session_path: "~/.lokai/chats/2".to_string(),
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:59Z")
                        .unwrap()
                        .into()
                },
                Conversation {
                    id: 3,
                    name: "conversation 3".to_string(),
                    session_path: "~/.lokai/chats/3".to_string(),
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:01:00Z")
                        .unwrap()
                        .into()
                },
                Conversation {
                    id: 4,
                    name: "conversation 4".to_string(),
                    session_path: "~/.lokai/chats/4".to_string(),
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:01:00Z")
                        .unwrap()
                        .into()
                },
            ]
        );

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql"))]
    async fn test_create_conversation(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);

        // when
        let mut transaction = pool.begin().await?;
        let new_conversation_1 =
            create_conversation(&mut *transaction, "conversation 1", "~/.lokai/chats/1").await?;
        let new_conversation_2 =
            create_conversation(&mut *transaction, "conversation 2", "~/.lokai/chats/1").await?;
        transaction.commit().await?;

        // then
        assert_eq!(table_count(&pool, "conversations").await?, 6);
        assert!(new_conversation_1.id < new_conversation_2.id);
        assert_eq!(new_conversation_1.name, "conversation 1");
        assert_eq!(new_conversation_2.name, "conversation 2");
        assert_eq!(new_conversation_1.created_at, new_conversation_2.created_at);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql", "../fixtures/messages.sql"))]
    async fn test_delete_conversation_cascade(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        // when
        let deleted_conversation = delete_conversation(&pool, 1).await?;

        // then
        assert_eq!(table_count(&pool, "conversations").await?, 3);
        assert_eq!(table_count(&pool, "messages").await?, 3);
        assert_eq!(
            deleted_conversation,
            Conversation {
                id: 1,
                name: "conversation 1".to_string(),
                session_path: "~/.lokai/chats/1".to_string(),
                created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:00Z")
                    .unwrap()
                    .into()
            }
        );

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql"))]
    async fn test_delete_conversation_that_doesnt_exist(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);

        // when
        let result = delete_conversation(&pool, 9999).await;

        // then
        assert!(result.is_err());
        assert_eq!(table_count(&pool, "conversations").await?, 4);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql"))]
    async fn test_get_messages_empty_table(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 0);

        // when
        let messages = get_messages(&pool, 1).await?;

        // then
        assert_eq!(messages, vec![]);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql", "../fixtures/messages.sql"))]
    async fn test_get_messages_non_empty_table(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        // when
        let messages = get_messages(&pool, 1).await?;

        // then
        assert_eq!(
            messages,
            vec![
                Message {
                    id: 1,
                    role: Role::User,
                    content: "why is the sky blue?".to_string(),
                    conversation_id: 1,
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:00Z")
                        .unwrap()
                        .into()
                },
                Message {
                    id: 2,
                    role: Role::Assistant,
                    content: "I don't know".to_string(),
                    conversation_id: 1,
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:59Z")
                        .unwrap()
                        .into()
                },
            ]
        );

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql", "../fixtures/messages.sql"))]
    async fn test_create_message(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        // when
        let mut transaction = pool.begin().await?;
        let new_message_1 =
            create_message(&mut *transaction, Role::User, "why is the sky blue?", 3).await?;
        let new_message_2 =
            create_message(&mut *transaction, Role::Assistant, "I don't know", 3).await?;
        transaction.commit().await?;

        // then
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 7);

        assert!(new_message_1.id < new_message_2.id);
        assert_eq!(new_message_1.created_at, new_message_2.created_at);

        assert_eq!(new_message_1.role, Role::User);
        assert_eq!(new_message_1.content, "why is the sky blue?".to_string());
        assert_eq!(new_message_1.conversation_id, 3);

        assert_eq!(new_message_2.role, Role::Assistant);
        assert_eq!(new_message_2.content, "I don't know".to_string());
        assert_eq!(new_message_2.conversation_id, 3);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql", "../fixtures/messages.sql"))]
    async fn test_create_message_conversation_doesnt_exist(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        // when
        let result = create_message(&pool, Role::User, "why is the sky blue?", 9999).await;

        // then
        assert!(result.is_err());
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql", "../fixtures/messages.sql"))]
    async fn test_update_message(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        let original_message = sqlx::query_as::<_, Message>(
            r#"
        SELECT *
        FROM messages
        WHERE id = ?1
            AND conversation_id = ?2
        "#,
        )
        .bind(2)
        .bind(1)
        .persistent(false)
        .fetch_one(&pool)
        .await?;

        assert_eq!(
            original_message,
            Message {
                id: 2,
                role: Role::Assistant,
                content: "I don't know".to_string(),
                conversation_id: 1,
                created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:59Z")
                    .unwrap()
                    .into()
            }
        );

        // when
        let updated_message = update_message(&pool, "Actually, I know why", 2).await?;

        // then
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        assert_eq!(original_message.id, updated_message.id);
        assert_eq!(original_message.role, updated_message.role);
        assert_ne!(original_message.content, updated_message.content);
        assert_eq!(original_message.content, "I don't know");
        assert_eq!(updated_message.content, "Actually, I know why");
        assert_eq!(
            original_message.conversation_id,
            updated_message.conversation_id
        );
        assert_eq!(original_message.created_at, updated_message.created_at);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql", "../fixtures/messages.sql"))]
    async fn test_update_message_that_doesnt_exist(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        // when
        let result = update_message(&pool, "Updated content", 9999).await;

        // then
        assert!(result.is_err());
        assert_eq!(table_count(&pool, "conversations").await?, 4);
        assert_eq!(table_count(&pool, "messages").await?, 5);

        Ok(())
    }
}
