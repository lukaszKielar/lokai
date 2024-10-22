use futures::StreamExt;
use kalosm::language::{Chat, Llama};
use sqlx::SqlitePool;
use tokio::sync::mpsc::{Receiver, UnboundedSender};

use crate::{
    db,
    event::{Event, InferenceType},
    models::{Message, Role},
    AppResult,
};

pub struct Assistant;

impl Assistant {
    pub fn run(
        llama: Llama,
        sqlite: SqlitePool,
        inference_rx: Receiver<Message>,
        event_tx: UnboundedSender<Event>,
    ) {
        tokio::spawn(async move {
            inference_stream(sqlite, inference_rx, event_tx, llama).await;
        });
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

        // TODO: create cache object that could keep different chats in memory for some time and load it when necessary
        let mut chat = Chat::builder(llama.clone())
            .with_try_session_path(&conversation.session_path)
            .build();
        let mut text_stream = chat.add_message(inference_message.content);

        let mut assistant_response =
            db::create_message(&sqlite, Role::Assistant, "", conversation.id).await?;

        while let Some(chunk) = text_stream.next().await {
            assistant_response.content.push_str(&chunk);

            // ignore send errors, I can at least wait until the end of assistant's response and save it to db
            // if the channel is closed we probably paniced anyway
            let _ = event_tx.send(Event::Inference(
                assistant_response.clone(),
                InferenceType::Streaming,
            ));
        }

        let assistant_response =
            db::update_message(&sqlite, &assistant_response.content, assistant_response.id).await?;
        chat.add_message(assistant_response.content);

        tokio::spawn(async move {
            match chat.save_session(&conversation.session_path).await {
                Ok(_) => tracing::info!("session saved to disk"),
                Err(err) => tracing::error!("Error while saving session: {}", err),
            }
        })
        .await?;
    }

    Ok(())
}
