use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::{sync::mpsc, task::JoinHandle};

use crate::{
    app::AppResult,
    crud::{create_message, get_messages},
    event::Event,
    models::{Message, Role},
};

static DEFAULT_LLM_MODEL: &str = "phi3:3.8b";
static OLLAMA_URL: &str = "http://host.docker.internal:11434";

pub struct Ollama {
    join_handle: JoinHandle<()>,
}

impl Ollama {
    pub fn new(
        sqlite: SqlitePool,
        inference_rx: mpsc::Receiver<Message>,
        event_tx: mpsc::UnboundedSender<Event>,
    ) -> Self {
        let join_handle = tokio::spawn(async move {
            let reqwest_client = reqwest::Client::new();
            inference(sqlite, inference_rx, event_tx, reqwest_client).await;
        });
        Self { join_handle }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OllamaMessage {
    pub role: Role,
    pub content: String,
}

impl From<Message> for OllamaMessage {
    fn from(value: Message) -> Self {
        Self {
            role: value.role,
            content: value.content,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OllamaChatResponse {
    pub message: OllamaMessage,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OllamaChatResponseStream {
    pub message: OllamaMessage,
    pub done: bool,
}

#[derive(Serialize, Debug)]
pub struct OllamaChatParams {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
    pub stream: bool,
}

impl OllamaChatParams {
    pub fn new<T: Into<OllamaMessage>>(model: String, messages: Vec<T>, stream: bool) -> Self {
        Self {
            model,
            messages: messages.into_iter().map(|elem| elem.into()).collect(),
            stream,
        }
    }
}

async fn inference(
    sqlite: SqlitePool,
    mut inference_rx: mpsc::Receiver<Message>,
    event_tx: mpsc::UnboundedSender<Event>,
    reqwest_client: reqwest::Client,
) -> AppResult<()> {
    while let Some(inference_message) = inference_rx.recv().await {
        let sqlite_clone = sqlite.clone();
        let transaction = sqlite_clone.begin().await?;

        let conversation_id = inference_message.conversation_id;
        let messages = get_messages(sqlite.clone(), conversation_id).await?;

        let params = OllamaChatParams::new(DEFAULT_LLM_MODEL.to_string(), messages, false);

        let response = reqwest_client
            .post(format!("{}/api/chat", OLLAMA_URL))
            .json(&params)
            .send()
            .await?
            .json::<OllamaChatResponse>()
            .await?;
        let assistant_response = create_message(
            sqlite.clone(),
            Role::Assistant,
            response.message.content,
            conversation_id,
        )
        .await?;

        transaction.commit().await?;

        event_tx.send(Event::Inference(assistant_response));
    }

    Ok(())
}
