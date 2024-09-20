use std::ops::{Deref, DerefMut};

use crossterm::event::KeyEvent;
use ratatui::widgets::{List, ListItem, ListState};
use sqlx::SqlitePool;
use tui_textarea::TextArea;

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

    pub fn delete_conversation(&mut self, conversation: Conversation) {
        if let Some(index) = self
            .conversations
            .iter()
            .position(|c| c.id == conversation.id)
        {
            self.conversations.remove(index);
            self.unselect();
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

// TODO: add internal attribute that will define error style and message
pub struct NewConversationPopup {
    text: Option<String>,
    text_area: TextArea<'static>,
    activated: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for NewConversationPopup {
    fn default() -> Self {
        Self {
            text: None,
            text_area: Default::default(),
            activated: Default::default(),
        }
    }
}

impl Deref for NewConversationPopup {
    type Target = TextArea<'static>;

    fn deref(&self) -> &Self::Target {
        &self.text_area
    }
}

impl DerefMut for NewConversationPopup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.text_area
    }
}

impl NewConversationPopup {
    pub fn clear(&mut self) {
        self.text_area.select_all();
        self.text_area.cut();
        self.text_area.set_yank_text("");
        self.text = None;
    }

    pub fn is_activated(&self) -> bool {
        self.activated
    }

    pub fn activate(&mut self) {
        self.activated = true;
        self.text = Some("".to_string());
    }

    pub fn deactivate(&mut self) {
        self.activated = false;
        self.clear();
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) {
        if !self.activated {
            panic!("Activate popup before you handle input")
        }

        self.text_area.input(key_event);
        self.text = Some(self.text_area.lines().join("\n").trim().to_string());
    }

    pub fn get_content(&self) -> Option<&String> {
        self.text.as_ref()
    }
}

enum YesOrNo {
    Yes,
    No,
}

pub struct DeleteConversationPopup {
    activated: bool,
    yes_or_no: YesOrNo,
}

#[allow(clippy::derivable_impls)]
impl Default for DeleteConversationPopup {
    fn default() -> Self {
        Self {
            activated: Default::default(),
            yes_or_no: YesOrNo::No,
        }
    }
}

impl DeleteConversationPopup {
    pub fn yes(&self) -> bool {
        match self.yes_or_no {
            YesOrNo::Yes => true,
            YesOrNo::No => false,
        }
    }

    pub fn is_activated(&self) -> bool {
        self.activated
    }

    pub fn activate(&mut self) {
        self.activated = true;
    }

    pub fn deactivate(&mut self) {
        self.activated = false;
    }

    pub fn confirm(&mut self) {
        self.yes_or_no = YesOrNo::Yes;
    }

    pub fn cancel(&mut self) {
        self.yes_or_no = YesOrNo::No;
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};

    use super::*;

    #[test]
    fn test_activate() {
        // given
        let mut popup = NewConversationPopup::default();

        // when
        popup.activate();

        // then
        assert!(popup.activated);
        assert_eq!(popup.text, Some("".to_string()));
    }

    #[test]
    fn test_deactivate() {
        // given
        let mut popup = NewConversationPopup::default();

        // when
        popup.deactivate();

        // then
        assert!(!popup.activated);
        assert_eq!(popup.text, None);
    }

    #[test]
    fn test_handle_input() {
        // given
        let mut popup = NewConversationPopup::default();
        popup.activate();

        // when
        popup.handle_input(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT));
        popup.handle_input(KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char(','), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('W'), KeyModifiers::SHIFT));
        popup.handle_input(KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char('!'), KeyModifiers::NONE));
        popup.handle_input(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE));

        // then
        assert_eq!(popup.text, Some("Hello, World!".to_string()));
    }

    #[test]
    #[should_panic]
    fn test_handle_input_not_activated() {
        // given
        let mut popup = NewConversationPopup::default();

        // when
        popup.handle_input(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT));
    }

    #[test]
    fn test_clear() {
        // given
        let mut popup = NewConversationPopup::default();
        popup.activate();
        popup.handle_input(KeyEvent::new(KeyCode::Char('H'), KeyModifiers::SHIFT));

        // when
        popup.clear();

        // then
        assert!(popup.activated);
        assert_eq!(popup.text, None);
    }
}
