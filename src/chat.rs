use ratatui::widgets::ListState;

use crate::models::Message;

// TODO: make all attrs private
// TODO: create common StatefulList trait and implement it for ConversationList and MessageList
#[derive(Default)]
pub struct Chat {
    pub messages: Vec<Message>,
    pub state: ListState,
}
