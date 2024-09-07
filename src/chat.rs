use ratatui::widgets::{List, ListItem, ListState};
use sqlx::SqlitePool;

use crate::{db, models::Message, AppResult};

pub struct Chat {
    messages: Vec<Message>,
    pub state: ListState,
    sqlite: SqlitePool,
}

impl Chat {
    pub fn new(sqlite: SqlitePool) -> Self {
        Self {
            messages: vec![],
            state: Default::default(),
            sqlite,
        }
    }

    pub fn currently_selected(&self) -> Option<Message> {
        let selected_index = self.state.selected()?;
        self.messages.get(selected_index).cloned()
    }

    pub fn reset(&mut self) {
        self.unselect();
        self.messages = vec![];
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn pop(&mut self) {
        self.messages.pop();
    }

    pub fn last(&self) -> Option<&Message> {
        self.messages.last()
    }

    pub fn up(&mut self) {
        self.state.scroll_up_by(1);
    }

    pub fn down(&mut self) {
        self.state.scroll_down_by(1);
    }

    pub async fn load_messages(&mut self, conversation_id: u32) -> AppResult<()> {
        let messages = db::get_messages(self.sqlite.clone(), conversation_id).await?;
        self.messages = messages;

        Ok(())
    }

    pub fn as_list_widget<F, T>(&self, f: F) -> List<'static>
    where
        F: Fn(&Message) -> T,
        T: Into<ListItem<'static>>,
    {
        let items = self
            .messages
            .iter()
            .map(|elem| f(elem).into())
            .collect::<Vec<ListItem>>();

        List::new(items)
    }
}
