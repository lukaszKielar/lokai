use ratatui::widgets::ListState;

use crate::models::Conversation;

// TODO: make all attrs private
// TODO: create common StatefulList trait and implement it for Conversations and MessageList
#[derive(Default)]
pub struct Conversations {
    pub conversations: Vec<Conversation>,
    pub state: ListState,
}

impl Conversations {
    pub fn currently_selected(&self) -> Option<Conversation> {
        let selected_index = self.state.selected()?;
        self.conversations.get(selected_index).cloned()
    }
}
