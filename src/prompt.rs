use tokio::sync::mpsc;
use tui_textarea::TextArea;

use crate::models::Message;

#[derive(Debug, Clone)]
pub struct Prompt<'a> {
    pub text_area: TextArea<'a>,
    pub inference_tx: mpsc::Sender<Message>,
}

impl<'a> Prompt<'a> {
    pub fn new(inference_tx: mpsc::Sender<Message>) -> Self {
        Self {
            text_area: Default::default(),
            inference_tx,
        }
    }

    pub fn clear(&mut self) {
        self.text_area.select_all();
        self.text_area.cut();
        self.text_area.set_yank_text("");
    }
}
