use std::error;

use ratatui::widgets::ListState;
use sqlx::SqlitePool;

use crate::config::Config;
use crate::db::get_conversations;
use crate::prompt::Prompt;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

// TODO: remove pub from attrs
// TODO: create common StatefulList trait and implement it for ConversationList and MessageList
// TODO: this should keep vec of Conversation (DB model)
#[derive(Debug, Default)]
pub struct ConversationList {
    pub items: Vec<String>,
    pub state: ListState,
}

// TODO: remove pub from attrs
// TODO: create common StatefulList trait and implement it for ConversationList and MessageList
// TODO: this should keep vec of Message (DB model)
#[derive(Debug, Default)]
pub struct MessageList {
    pub items: Vec<String>,
    pub state: ListState,
}

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
pub struct App {
    pub sqlite: Option<SqlitePool>,
    pub running: bool,
    _config: Config,
    pub conversation_list: ConversationList,
    pub message_list: MessageList,
    focus: AppFocus,
    pub prompt: Prompt<'static>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            sqlite: None,
            running: true,
            _config: Default::default(),
            conversation_list: Default::default(),
            message_list: Default::default(),
            focus: Default::default(),
            prompt: Default::default(),
        }
    }
}

impl App {
    pub async fn init(&mut self, sqlite: SqlitePool) -> AppResult<()> {
        self.sqlite.replace(sqlite);
        let conversations = get_conversations(self.sqlite.as_ref().unwrap().clone())
            .await?
            .iter()
            .map(|elem| elem.name.to_owned())
            .collect();
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

    pub async fn down_conversation(&mut self) {
        self.conversation_list.state.scroll_down_by(1);
    }
}
