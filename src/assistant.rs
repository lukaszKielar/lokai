use futures::StreamExt;
use kalosm::language::{Chat, Llama};
use sqlx::SqlitePool;
use tokio::{
    sync::mpsc::{Receiver, UnboundedSender},
    task::JoinHandle,
};

use crate::{
    db,
    event::{Event, InferenceType},
    models::Message,
    AppResult,
};

pub struct Assistant {
    _join_handle: JoinHandle<()>,
}

impl Assistant {
    pub fn new(
        llama: Llama,
        sqlite: SqlitePool,
        inference_rx: Receiver<Message>,
        event_tx: UnboundedSender<Event>,
    ) -> Self {
        let join_handle = tokio::spawn(async move {
            inference_stream(sqlite, inference_rx, event_tx, llama).await;
        });

        Self {
            _join_handle: join_handle,
        }
    }
}

async fn inference_stream(
    sqlite: SqlitePool,
    mut inference_rx: Receiver<Message>,
    event_tx: UnboundedSender<Event>,
    llama: Llama,
) -> AppResult<()> {
    while let Some(inference_message) = inference_rx.recv().await {
        let conversation = db::get_conversation(&sqlite, inference_message.conversation_id).await?;
        let mut chat = Chat::new(llama.clone());
        let mut text_stream = chat.add_message(inference_message.content);

        let mut assistant_response = Message::assistant(String::new(), conversation.id);

        while let Some(chunk) = text_stream.next().await {
            assistant_response.content.push_str(&chunk);

            event_tx.send(Event::Inference(
                assistant_response.clone(),
                InferenceType::Streaming,
            ))?;
        }
    }

    Ok(())
}
