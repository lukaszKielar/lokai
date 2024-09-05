use sqlx::SqlitePool;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::{
    chat::Chat, conversations::Conversations, db::get_conversations, event::Event, models::Message,
    ollama::Ollama, prompt::Prompt, AppResult,
};

#[derive(Copy, Clone)]
pub enum AppFocus {
    Conversation,
    Messages,
    Prompt,
}

impl Default for AppFocus {
    fn default() -> Self {
        Self::Conversation
    }
}

impl AppFocus {
    pub fn next(&self) -> AppFocus {
        match self {
            AppFocus::Conversation => AppFocus::Messages,
            AppFocus::Messages => AppFocus::Prompt,
            AppFocus::Prompt => AppFocus::Conversation,
        }
    }

    pub fn previous(&self) -> AppFocus {
        match self {
            AppFocus::Conversation => AppFocus::Prompt,
            AppFocus::Messages => AppFocus::Conversation,
            AppFocus::Prompt => AppFocus::Messages,
        }
    }
}

// TODO: remove all pub attributes
pub struct App {
    pub sqlite: SqlitePool,
    pub running: bool,
    pub conversations: Conversations,
    pub chat: Chat,
    focus: AppFocus,
    pub prompt: Prompt<'static>,
    pub ollama: Ollama,
}

impl App {
    pub fn new(sqlite: SqlitePool, event_tx: UnboundedSender<Event>) -> Self {
        let (inference_tx, inference_rx) = mpsc::channel::<Message>(10);
        Self {
            sqlite: sqlite.clone(),
            running: true,
            conversations: Default::default(),
            chat: Default::default(),
            focus: Default::default(),
            prompt: Prompt::new(inference_tx),
            ollama: Ollama::new(sqlite, inference_rx, event_tx),
        }
    }

    pub async fn init(&mut self) -> AppResult<()> {
        let conversations = get_conversations(self.sqlite.clone()).await?;
        self.conversations.conversations = conversations;

        Ok(())
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn current_focus(&self) -> AppFocus {
        self.focus
    }

    pub fn next_focus(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn previous_focus(&mut self) {
        self.focus = self.focus.previous();
    }
}
