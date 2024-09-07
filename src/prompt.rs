use std::ops::Deref;

use crossterm::event::KeyEvent;
use ratatui::widgets::Block;
use tui_textarea::TextArea;

#[derive(Debug, Clone, Default)]
pub struct Prompt {
    text_area: TextArea<'static>,
}

impl Prompt {
    pub fn clear(&mut self) {
        self.text_area.select_all();
        self.text_area.cut();
        self.text_area.set_yank_text("");
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        self.text_area.input(key_event);
    }

    pub fn new_line(&mut self) {
        self.text_area.insert_str("\n");
    }

    pub fn get_content(&self) -> String {
        self.text_area.lines().join("\n").trim().to_string()
    }

    pub fn set_block(&mut self, block: Block<'static>) {
        self.text_area.set_block(block);
    }
}

impl Deref for Prompt {
    type Target = TextArea<'static>;

    fn deref(&self) -> &Self::Target {
        &self.text_area
    }
}
