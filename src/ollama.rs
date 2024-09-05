use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::{
    sync::mpsc::{Receiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    db::{create_message, get_messages, update_message},
    event::{Event, InferenceType},
    models::{Message, Role},
    AppResult,
};

static DEFAULT_LLM_MODEL: &str = "phi3:3.8b";
static OLLAMA_URL: &str = "http://host.docker.internal:11434";

pub struct Ollama {
    _join_handle: JoinHandle<()>,
}

impl Ollama {
    pub fn new(
        sqlite: SqlitePool,
        inference_rx: Receiver<Message>,
        event_tx: UnboundedSender<Event>,
    ) -> Self {
        let join_handle = tokio::spawn(async move {
            let reqwest_client = reqwest::Client::new();
            let _ = inference_stream(sqlite, inference_rx, event_tx, reqwest_client).await;
        });
        Self {
            _join_handle: join_handle,
        }
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

#[allow(dead_code)]
async fn inference(
    sqlite: SqlitePool,
    mut inference_rx: Receiver<Message>,
    event_tx: UnboundedSender<Event>,
    reqwest_client: reqwest::Client,
) -> AppResult<()> {
    while let Some(inference_message) = inference_rx.recv().await {
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

        let content = response.message.content.trim().to_string();

        let assistant_response =
            create_message(sqlite.clone(), Role::Assistant, content, conversation_id).await?;

        let _ = event_tx.send(Event::Inference(
            assistant_response,
            InferenceType::NonStreaming,
        ));
    }

    Ok(())
}

async fn inference_stream(
    sqlite: SqlitePool,
    mut inference_rx: Receiver<Message>,
    event_tx: UnboundedSender<Event>,
    reqwest_client: reqwest::Client,
) -> AppResult<()> {
    while let Some(inference_message) = inference_rx.recv().await {
        let conversation_id = inference_message.conversation_id;
        let messages = get_messages(sqlite.clone(), conversation_id).await?;

        let params = OllamaChatParams::new(DEFAULT_LLM_MODEL.to_string(), messages, true);

        let mut stream = reqwest_client
            .post(format!("{}/api/chat", OLLAMA_URL))
            .json(&params)
            .send()
            .await?
            .bytes_stream()
            .map(|chunk| chunk.unwrap())
            .map(|chunk| serde_json::from_slice::<OllamaChatResponseStream>(&chunk));

        let assistant_response = create_message(
            sqlite.clone(),
            Role::Assistant,
            "".to_string(),
            conversation_id,
        )
        .await?;

        let mut is_first_chunk = true;
        let mut content = String::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(response) => {
                    if response.done {
                        break;
                    }

                    content.push_str(&response.message.content);
                    if is_first_chunk {
                        is_first_chunk = false;
                        content = content.trim_start().to_string();
                    };

                    {
                        let mut assistant_response = assistant_response.clone();
                        assistant_response.content = content.clone();

                        event_tx.send(Event::Inference(
                            assistant_response,
                            InferenceType::Streaming,
                        ))?;
                    }
                }
                Err(_) => return Err(format!("Error while reading chunk [{:?}]", chunk).into()),
            }
        }

        update_message(sqlite.clone(), content, assistant_response.id).await?;
    }

    Ok(())
}
