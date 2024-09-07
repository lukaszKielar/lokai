use ratatui::widgets::{List, ListItem, ListState};
use sqlx::SqlitePool;

use crate::models::Conversation;

pub struct Conversations {
    conversations: Vec<Conversation>,
    pub state: ListState,
    _sqlite: SqlitePool,
}

impl Conversations {
    pub fn new(sqlite: SqlitePool) -> Self {
        Self {
            conversations: vec![],
            state: Default::default(),
            _sqlite: sqlite,
        }
    }

    pub fn set_conversations(&mut self, conversations: Vec<Conversation>) {
        self.conversations = conversations;
    }

    pub fn currently_selected(&self) -> Option<Conversation> {
        let selected_index = self.state.selected()?;
        self.conversations.get(selected_index).cloned()
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn push(&mut self, conversation: Conversation) {
        self.conversations.push(conversation);
    }

    pub fn up(&mut self) {
        self.state.scroll_up_by(1);
    }

    pub fn down(&mut self) {
        self.state.scroll_down_by(1);
    }

    pub fn as_list_widget<F, T>(&self, f: F) -> List<'static>
    where
        F: Fn(&Conversation) -> T,
        T: Into<ListItem<'static>>,
    {
        let items = self
            .conversations
            .iter()
            .map(|elem| f(elem).into())
            .collect::<Vec<ListItem>>();

        List::new(items)
    }
}
