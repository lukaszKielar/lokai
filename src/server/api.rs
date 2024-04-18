use crate::models::Message;
use leptos::{server, ServerFnError};
use uuid::Uuid;

// TODO: save every prompt, response and context to database, async thread
// TODO: this function should take id of the conversation, prompt and context (history of conversation)
#[server(Chat, "/api")]
pub async fn chat(user_message: Message) -> Result<Message, ServerFnError> {
    use super::{db, ollama::*};
    use leptos::use_context;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let conversation_id = user_message.conversation_id;

    // TODO: that should be different call to server
    {
        let db_pool = db_pool.clone();
        let _ = db::create_message(db_pool, user_message).await;
    }

    let messages = {
        let db_pool = db_pool.clone();
        db::get_conversation_messages(db_pool, conversation_id)
            .await
            // TODO: handle result
            .unwrap()
            .into_iter()
            .map(|m| OllamaMessage::from(m))
            .collect()
    };

    // TODO: handle lack of context
    let client = use_context::<reqwest::Client>().expect("reqwest.Client not found");
    let params = OllamaChatParams {
        model: default_model(),
        messages: messages,
        stream: false,
    };

    // TODO: properly handle errors
    let response: OllamaChatResponse = client
        .post("http://host.docker.internal:11434/api/chat")
        .json(&params)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let assistant_message = Message::assistant(response.message.content, conversation_id);

    {
        let assistant_message = assistant_message.clone();
        tokio::spawn(db::create_message(db_pool, assistant_message));
    }

    Ok(assistant_message)
}

#[server(CreateMessage, "/api")]
pub async fn create_message(user_message: Message) -> Result<Message, ServerFnError> {
    use super::db;
    use leptos::logging;
    use leptos::use_context;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");
    let _ = db::create_message(db_pool, user_message.clone())
        .await
        .unwrap();

    Ok(user_message)
}

#[server(GetConversationMessages, "/api")]
pub async fn get_conversation_messages(
    conversation_id: Uuid,
) -> Result<Vec<Message>, ServerFnError> {
    use super::db;
    use leptos::use_context;
    use sqlx::SqlitePool;

    let db_pool = use_context::<SqlitePool>().expect("SqlitePool not found");

    let messages = db::get_conversation_messages(db_pool, conversation_id)
        .await
        .unwrap();

    Ok(messages)
}