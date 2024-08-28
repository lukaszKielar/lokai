use std::error;

use ratatui::widgets::ListState;

use crate::config::Config;

pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub struct ConversationList {
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
    pub running: bool,
    _config: Config,
    pub conversation_list: ConversationList,
    focus: AppFocus,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            _config: Default::default(),
            conversation_list: ConversationList {
                items: vec!["conversation 1".to_string(), "conversation 2".to_string()],
                state: Default::default(),
            },
            focus: Default::default(),
        }
    }
}

impl App {
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
