use std::error;

use ratatui::widgets::ListState;
use sqlx::SqlitePool;
use tokio::sync::mpsc;

use crate::db::get_conversations;
use crate::event::Event;
use crate::models::{Conversation, Message};
use crate::ollama::Ollama;
use crate::prompt::Prompt;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// TODO: remove pub from attrs
// TODO: create common StatefulList trait and implement it for ConversationList and MessageList
#[derive(Default)]
pub struct ConversationList {
    pub items: Vec<Conversation>,
    pub state: ListState,
}

impl ConversationList {
    pub fn currently_selected(&self) -> Option<Conversation> {
        let selected_index = self.state.selected()?;
        self.items.get(selected_index).cloned()
    }
}

// TODO: remove pub from attrs
// TODO: create common StatefulList trait and implement it for ConversationList and MessageList
#[derive(Default)]
pub struct MessageList {
    pub items: Vec<Message>,
    pub state: ListState,
}

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

pub struct App {
    pub sqlite: SqlitePool,
    pub running: bool,
    pub conversation_list: ConversationList,
    pub message_list: MessageList,
    focus: AppFocus,
    pub prompt: Prompt<'static>,
    pub ollama: Ollama,
}

impl App {
    pub fn new(sqlite: SqlitePool, event_tx: mpsc::UnboundedSender<Event>) -> Self {
        let (inference_tx, inference_rx) = mpsc::channel::<Message>(10);
        Self {
            sqlite: sqlite.clone(),
            running: true,
            conversation_list: Default::default(),
            message_list: Default::default(),
            focus: Default::default(),
            prompt: Prompt::new(inference_tx),
            ollama: Ollama::new(sqlite, inference_rx, event_tx),
        }
    }

    pub async fn init(&mut self) -> AppResult<()> {
        let conversations = get_conversations(self.sqlite.clone()).await?;
        self.conversation_list.items = conversations;

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
