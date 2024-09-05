use ratatui::widgets::ListState;

use crate::models::Message;

// TODO: remove pub from attrs
// TODO: create common StatefulList trait and implement it for ConversationList and MessageList
#[derive(Default)]
pub struct Chat {
    pub messages: Vec<Message>,
    pub state: ListState,
}
