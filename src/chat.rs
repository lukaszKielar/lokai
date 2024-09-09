use ratatui::widgets::{List, ListItem, Paragraph, ScrollbarState};
use sqlx::SqlitePool;

use crate::{db, models::Message, AppResult};

pub struct Chat {
    messages: Vec<Message>,
    pub vertical_scrollbar_state: ScrollbarState,
    pub vertical_scroll: usize,
    vertical_scrollbar_content_length: usize,
    sqlite: SqlitePool,
}

impl Chat {
    pub fn new(sqlite: SqlitePool) -> Self {
        Self {
            messages: vec![],
            vertical_scrollbar_state: Default::default(),
            vertical_scroll: Default::default(),
            vertical_scrollbar_content_length: Default::default(),
            sqlite,
        }
    }

    pub fn reset(&mut self) {
        self.messages = vec![];
        self.vertical_scroll = 0;
        self.vertical_scrollbar_state =
            self.vertical_scrollbar_state.position(self.vertical_scroll);
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
            if next_position > self.vertical_scrollbar_content_length {
                self.vertical_scrollbar_content_length
            } else {
                next_position
            }
        };
        self.vertical_scrollbar_state =
            self.vertical_scrollbar_state.position(self.vertical_scroll);
    }

    pub async fn load_messages(&mut self, conversation_id: u32) -> AppResult<()> {
        self.reset();

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

    pub fn as_paragraph<F>(&mut self, f: F, area_height: usize) -> Paragraph<'static>
    where
        F: Fn(&Message) -> String,
    {
        let text = self.messages.iter().map(f).collect::<Vec<_>>().join("\n");

        self.vertical_scrollbar_content_length =
            calculate_vertical_scrollbar_content_length(&text, area_height);

        self.vertical_scrollbar_state = self
            .vertical_scrollbar_state
            .content_length(self.vertical_scrollbar_content_length);

        Paragraph::new(text)
    }
}

fn calculate_vertical_scrollbar_content_length(text: &str, area_height: usize) -> usize {
    let lines_of_text = text.lines().collect::<Vec<_>>().len();
    // area has a border which takes 2 additional lines
    let area_height = area_height - 2;

    if lines_of_text > area_height {
        (lines_of_text - area_height) + 2
    } else {
        0
    }
}
