use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use sqlx::SqlitePool;
use tokio::sync::mpsc::{self, Sender, UnboundedSender};

use crate::{
    chat::Chat,
    conversations::{Conversations, DeleteConversationPopup, NewConversationPopup},
    db,
    event::{Event, InferenceType},
    models::{Message, Role},
    ollama::Ollama,
    prompt::Prompt,
    AppResult,
};

#[derive(Copy, Clone)]
pub enum AppFocus {
    Conversation,
    Messages,
    Prompt,
}

impl Default for AppFocus {
    fn default() -> Self {
        Self::Conversation
    }
}

impl AppFocus {
    pub fn next(&self) -> AppFocus {
        match self {
            AppFocus::Conversation => AppFocus::Messages,
            AppFocus::Messages => AppFocus::Prompt,
            AppFocus::Prompt => AppFocus::Conversation,
        }
    }

    pub fn previous(&self) -> AppFocus {
        match self {
            AppFocus::Conversation => AppFocus::Prompt,
            AppFocus::Messages => AppFocus::Conversation,
            AppFocus::Prompt => AppFocus::Messages,
        }
    }
}

// TODO: create shared AppState(SqlitePool)

pub struct App {
    pub chat: Chat,
    pub conversations: Conversations,
    pub prompt: Prompt,
    // TODO: currently both popups can co-exist, it has to change
    // there can be one popup at a time
    // TODO: I cannot allow empty conversation
    pub new_conversation_popup: NewConversationPopup,
    pub delete_conversation_popup: DeleteConversationPopup,
    focus: AppFocus,
    event_tx: UnboundedSender<Event>,
    inference_tx: Sender<Message>,
    running: bool,
    sqlite: SqlitePool,
    _ollama: Ollama,
}

impl App {
    pub fn new(sqlite: SqlitePool, event_tx: UnboundedSender<Event>) -> Self {
        let (inference_tx, inference_rx) = mpsc::channel::<Message>(10);
        Self {
            chat: Chat::new(sqlite.clone()),
            conversations: Conversations::new(sqlite.clone()),
            prompt: Default::default(),
            new_conversation_popup: Default::default(),
            delete_conversation_popup: Default::default(),
            focus: Default::default(),
            event_tx: event_tx.clone(),
            inference_tx,
            running: true,
            sqlite: sqlite.clone(),
            _ollama: Ollama::new(sqlite, inference_rx, event_tx),
        }
    }

    pub async fn init(&mut self) -> AppResult<()> {
        let conversations = db::get_conversations(&self.sqlite).await?;
        self.conversations.set_conversations(conversations);

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn current_focus(&self) -> AppFocus {
        self.focus
    }

    pub fn next_focus(&mut self) {
        self.focus = self.focus.next();
    }

    pub fn previous_focus(&mut self) {
        self.focus = self.focus.previous();
    }

    pub async fn handle_key_events(&mut self, key_event: KeyEvent) -> AppResult<()> {
        match key_event.code {
            // Ctrl + c -> exit
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.running = false;

                    return Ok(());
                }

                if self.new_conversation_popup.is_activated() {
                    self.new_conversation_popup.handle_input(key_event);

                    return Ok(());
                }

                if let AppFocus::Prompt = self.current_focus() {
                    self.prompt.handle_input(key_event);
                }
            }
            // Ctrl + n
            KeyCode::Char('n') | KeyCode::Char('N') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.new_conversation_popup.activate();

                    return Ok(());
                }

                if self.delete_conversation_popup.is_activated() {
                    self.delete_conversation_popup.deactivate();

                    return Ok(());
                }

                if self.new_conversation_popup.is_activated() {
                    self.new_conversation_popup.handle_input(key_event);

                    return Ok(());
                }

                if let AppFocus::Prompt = self.current_focus() {
                    self.prompt.handle_input(key_event);
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if self.delete_conversation_popup.is_activated() {
                    if let Some(conversation) = self.conversations.currently_selected() {
                        db::delete_conversation(&self.sqlite, conversation.id).await?;
                        self.conversations.delete_conversation(conversation);
                        self.chat.reset();
                    }
                    // we deactivate anyway, if there was conversation it's gone now
                    // if there wasn't any, no need to keep popup anymore
                    self.delete_conversation_popup.deactivate();

                    return Ok(());
                }

                if self.new_conversation_popup.is_activated() {
                    self.new_conversation_popup.handle_input(key_event);

                    return Ok(());
                }

                if let AppFocus::Prompt = self.current_focus() {
                    self.prompt.handle_input(key_event);
                }
            }
            KeyCode::Enter => {
                if self.new_conversation_popup.is_activated() {
                    // TODO: add here Option<Error> that will hold error message for draw function
                    // e.g. when content of the popup is an empty string I want to show to the user
                    // message that it cannot be null, as well as use red color to indicate problem
                    if let Some(conversation_name) = self.new_conversation_popup.get_content() {
                        let new_conversation =
                            db::create_conversation(&self.sqlite, conversation_name).await?;
                        self.conversations.push(new_conversation);
                        self.new_conversation_popup.deactivate();
                    }

                    return Ok(());
                }

                // enter triggers deletion - this is default behaviour
                if self.delete_conversation_popup.is_activated() {
                    if let Some(conversation) = self.conversations.currently_selected() {
                        db::delete_conversation(&self.sqlite, conversation.id).await?;
                        self.conversations.delete_conversation(conversation);
                        self.chat.reset();
                    }
                    self.delete_conversation_popup.deactivate();

                    return Ok(());
                }

                // NOTE: crossterm currently cannot recognise combination of Enter+Shift.
                // KeyEvent.modifiers are not properly registered, so Enter+Shift is seen as regular Enter.
                // https://github.com/crossterm-rs/crossterm/issues/685
                if let AppFocus::Prompt = self.current_focus() {
                    if key_event.modifiers == KeyModifiers::SHIFT {
                        self.prompt.new_line();
                    } else {
                        // we're able to send only when we have selected conversation
                        if let Some(conversation) = self.conversations.currently_selected() {
                            let user_prompt = self.prompt.get_content();
                            let user_message = db::create_message(
                                &self.sqlite,
                                Role::User,
                                &user_prompt,
                                conversation.id,
                            )
                            .await?;
                            self.chat.push_message(user_message.clone());
                            self.inference_tx.send(user_message).await?;
                            self.prompt.clear();
                        }
                    }
                }
            }
            KeyCode::Down => match self.current_focus() {
                AppFocus::Conversation => {
                    // TODO: fix screen flickering when currently selected conversation is on the bottom of the screen
                    // I don't need to reload messages again, as I cannot select any other conversation, there's nothing after
                    self.conversations.down();
                    if let Some(conversation) = self.conversations.currently_selected() {
                        self.chat.load_messages(conversation.id).await?;
                        // I could've call self.chat.scroll_to_bottom, but at the time of reseting chat
                        // I had lost all information about scrollbar
                        // I'll get it next time my UI recalculates scrollbar's params and updates self.chat state
                        // We know that event we send below will happen after that, therefore it's safe to do it
                        self.event_tx.send(Event::ChatBottomScroll)?;
                    }
                }
                AppFocus::Messages => self.chat.scroll_down(),
                AppFocus::Prompt => {
                    self.prompt.handle_input(key_event);
                }
            },
            KeyCode::Up => match self.current_focus() {
                AppFocus::Conversation => {
                    // TODO: fix screen flickering when currently selected conversation is on top of the screen
                    // I don't need to reload messages again, as I cannot select any other conversation, there's nothing before
                    self.conversations.up();
                    if let Some(conversation) = self.conversations.currently_selected() {
                        self.chat.load_messages(conversation.id).await?;
                        // I could've call self.chat.scroll_to_bottom, but at the time of reseting chat
                        // I had lost all information about scrollbar
                        // I'll get it next time my UI recalculates scrollbar's params and updates self.chat state
                        // We know that event we send below will happen after that, therefore it's safe to do it
                        self.event_tx.send(Event::ChatBottomScroll)?;
                    }
                }
                AppFocus::Messages => self.chat.scroll_up(),
                AppFocus::Prompt => {
                    self.prompt.handle_input(key_event);
                }
            },
            KeyCode::Esc => {
                if self.new_conversation_popup.is_activated() {
                    self.new_conversation_popup.deactivate();

                    return Ok(());
                }

                if self.delete_conversation_popup.is_activated() {
                    self.delete_conversation_popup.deactivate();

                    return Ok(());
                }

                if let AppFocus::Conversation = self.current_focus() {
                    self.conversations.unselect();
                    self.chat.reset();
                }
            }
            KeyCode::Tab => {
                if !self.new_conversation_popup.is_activated() {
                    self.next_focus()
                }
            }
            KeyCode::BackTab => {
                if !self.new_conversation_popup.is_activated() {
                    self.previous_focus()
                }
            }
            KeyCode::Delete => {
                if let AppFocus::Conversation = self.current_focus() {
                    if self.conversations.currently_selected().is_some() {
                        if self.new_conversation_popup.is_activated() {
                            self.new_conversation_popup.deactivate();
                        }

                        if !self.delete_conversation_popup.is_activated() {
                            self.delete_conversation_popup.activate();
                        }
                    }

                    return Ok(());
                }
            }
            _ => {
                if self.new_conversation_popup.is_activated() {
                    self.new_conversation_popup.handle_input(key_event);

                    return Ok(());
                }

                if let AppFocus::Prompt = self.current_focus() {
                    self.prompt.handle_input(key_event);
                }
            }
        }
        Ok(())
    }

    async fn handle_inference_event(&mut self, message: Message) -> AppResult<()> {
        self.chat.push_message(message);

        Ok(())
    }

    async fn handle_inference_stream_event(&mut self, message: Message) -> AppResult<()> {
        if let Some(last_message) = self.chat.get_last_message() {
            if let Some(conversation) = self.conversations.currently_selected() {
                if conversation.id.eq(&message.conversation_id) {
                    match last_message.role {
                        Role::Assistant => {
                            self.chat.pop_message();
                            self.chat.push_message(message);
                        }
                        Role::System => {}
                        Role::User => {
                            self.chat.push_message(message);
                        }
                    }
                }
            }
        };

        Ok(())
    }

    async fn handle_chat_bottom_scroll_event(&mut self) -> AppResult<()> {
        self.chat.scroll_to_bottom();

        Ok(())
    }

    async fn handle_prompt_transcription(&mut self, word: String) -> AppResult<()> {
        self.prompt.insert_str(word);

        Ok(())
    }

    pub async fn handle_events(&mut self, event: Event) -> AppResult<()> {
        match event {
            Event::TerminalTick => Ok(()),
            Event::Key(key_event) => self.handle_key_events(key_event).await,
            Event::Inference(message, InferenceType::Streaming) => {
                self.handle_inference_stream_event(message).await
            }
            Event::Inference(message, InferenceType::NonStreaming) => {
                self.handle_inference_event(message).await
            }
            Event::ChatBottomScroll => self.handle_chat_bottom_scroll_event().await,
            Event::PromptTranscription(word) => self.handle_prompt_transcription(word).await,
        }
    }
}
