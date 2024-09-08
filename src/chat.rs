use ratatui::widgets::{List, ListItem, Paragraph, ScrollbarState};
use sqlx::SqlitePool;

use crate::{db, models::Message, AppResult};

pub struct Chat {
    messages: Vec<Message>,
    pub vertical_scrollbar_state: ScrollbarState,
    pub vertical_scroll: usize,
    sqlite: SqlitePool,
}

impl Chat {
    pub fn new(sqlite: SqlitePool) -> Self {
        Self {
            messages: vec![],
            vertical_scrollbar_state: Default::default(),
            vertical_scroll: Default::default(),
            sqlite,
        }
    }

    pub fn reset(&mut self) {
        self.messages = vec![];
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
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.vertical_scrollbar_state =
            self.vertical_scrollbar_state.position(self.vertical_scroll);
    }

    pub fn down(&mut self) {
        self.vertical_scroll = {
            let next_position = self.vertical_scroll.saturating_add(1);
            if next_position > self.messages.len() {
                self.messages.len()
            } else {
                next_position
            }
        };
        self.vertical_scrollbar_state =
            self.vertical_scrollbar_state.position(self.vertical_scroll);
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

    pub fn as_paragraph<F>(&mut self, f: F) -> Paragraph<'static>
    where
        F: Fn(&Message) -> String,
    {
        let text = self.messages.iter().map(&f).collect::<Vec<_>>().join("\n");

        self.vertical_scrollbar_state = self
            .vertical_scrollbar_state
            .content_length(self.messages.len());

        Paragraph::new(text)
    }
}
