use sqlx::{Executor, Sqlite};

use crate::{models::Conversation, AppResult};

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

pub async fn create_conversation<'e, E>(executor: E, name: &str) -> AppResult<Conversation>
where
    E: Executor<'e, Database = Sqlite>,
{
    let conversation = sqlx::query_as(
        r#"
        INSERT INTO conversations(name) VALUES (?1)
        RETURNING *
        "#,
    )
    .bind(name)
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

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use sqlx::{Row, SqlitePool};

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
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:00Z")
                        .unwrap()
                        .into()
                },
                Conversation {
                    id: 2,
                    name: "conversation 2".to_string(),
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:00:59Z")
                        .unwrap()
                        .into()
                },
                Conversation {
                    id: 3,
                    name: "conversation 3".to_string(),
                    created_at: DateTime::parse_from_rfc3339("2024-09-13T09:01:00Z")
                        .unwrap()
                        .into()
                },
                Conversation {
                    id: 4,
                    name: "conversation 4".to_string(),
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
        let new_conversation_1 = create_conversation(&mut *transaction, "conversation 1").await?;
        let new_conversation_2 = create_conversation(&mut *transaction, "conversation 2").await?;
        transaction.commit().await?;

        // then
        assert_eq!(table_count(&pool, "conversations").await?, 6);
        assert!(new_conversation_1.id < new_conversation_2.id);
        assert_eq!(new_conversation_1.name, "conversation 1");
        assert_eq!(new_conversation_2.name, "conversation 2");
        assert_eq!(new_conversation_1.created_at, new_conversation_2.created_at);

        Ok(())
    }

    #[sqlx::test(fixtures("../fixtures/conversations.sql"))]
    async fn test_delete_conversation_cascade(pool: SqlitePool) -> AppResult<()> {
        // given
        assert_eq!(table_count(&pool, "conversations").await?, 4);

        // when
        let deleted_conversation = delete_conversation(&pool, 1).await?;

        // then
        assert_eq!(table_count(&pool, "conversations").await?, 3);
        assert_eq!(
            deleted_conversation,
            Conversation {
                id: 1,
                name: "conversation 1".to_string(),
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
}
