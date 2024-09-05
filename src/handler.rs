use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::{
    app::{App, AppFocus},
    db::{create_message, get_messages},
    models::{Message, Role},
    AppResult,
};

/// Some key events are associated with specific focus blocks, other events work globally
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Ctrl + c -> exit
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            } else {
                app.prompt.text_area.input(key_event);
            }
        }
        KeyCode::Enter => {
            // NOTE: crossterm currently cannot recognise combination of Enter+Shift.
            // KeyEvent.modifiers are not properly registered, so Enter+Shift is seen as regular Enter.
            // https://github.com/crossterm-rs/crossterm/issues/685
            if let AppFocus::Prompt = app.current_focus() {
                if key_event.modifiers == KeyModifiers::SHIFT {
                    app.prompt.text_area.input(key_event);
                } else {
                    // we're able to send only when we have selected conversation
                    if let Some(conversation) = app.conversations.currently_selected() {
                        // TODO: 1. get text, 2. send text to inference thread, 3. clear input
                        let user_input = app.prompt.text_area.lines().join("\n").trim().to_string();
                        let user_message = create_message(
                            app.sqlite.clone(),
                            Role::User,
                            user_input,
                            conversation.id,
                        )
                        .await?;
                        app.chat.messages.push(user_message.clone());
                        app.prompt.inference_tx.send(user_message).await?;
                        app.prompt.clear();
                    }
                }
            }
        }
        KeyCode::Down => match app.current_focus() {
            AppFocus::Conversation => {
                // 1. get index of conversation
                // 2. get messages for conversation
                // 3. mutate state of app by assigning messages to proper attr
                app.conversations.state.scroll_down_by(1);
                if let Some(current_index) = app.conversations.state.selected() {
                    app.chat.state.select(None);
                    if let Some(item) = app.conversations.conversations.get(current_index) {
                        let messages = get_messages(app.sqlite.clone(), item.id).await?;
                        app.chat.messages = messages;
                    }
                };
            }
            AppFocus::Messages => app.chat.state.scroll_down_by(1),
            AppFocus::Prompt => {
                app.prompt.text_area.input(key_event);
            }
        },
        KeyCode::Up => match app.current_focus() {
            AppFocus::Conversation => {
                app.conversations.state.scroll_up_by(1);
                if let Some(current_index) = app.conversations.state.selected() {
                    if let Some(item) = app.conversations.conversations.get(current_index) {
                        let messages = get_messages(app.sqlite.clone(), item.id).await?;
                        app.chat.messages = messages;
                    }
                };
            }
            AppFocus::Messages => app.chat.state.scroll_up_by(1),
            AppFocus::Prompt => {
                app.prompt.text_area.input(key_event);
            }
        },
        KeyCode::Esc => match app.current_focus() {
            AppFocus::Conversation => {
                app.conversations.state.select(None);
                app.chat.messages = vec![];
            }
            AppFocus::Messages => app.chat.state.select(None),
            AppFocus::Prompt => {}
        },
        KeyCode::Tab => app.next_focus(),
        KeyCode::BackTab => app.previous_focus(),
        _ => {
            if let AppFocus::Prompt = app.current_focus() {
                app.prompt.text_area.input(key_event);
            }
        }
    }
    Ok(())
}

pub fn handle_inference_event(message: Message, app: &mut App) -> AppResult<()> {
    app.chat.messages.push(message);

    Ok(())
}

// TODO: simply ignore inference when message.conversation_id doesn't match with actually selected conversation
pub fn handle_inference_stream_event(message: Message, app: &mut App) -> AppResult<()> {
    if let Some(last_message) = app.chat.messages.last() {
        if let Some(conversation) = app.conversations.currently_selected() {
            if conversation.id.eq(&message.conversation_id) {
                match last_message.role {
                    Role::Assistant => {
                        app.chat.messages.pop();
                        app.chat.messages.push(message);
                    }
                    Role::System => {}
                    Role::User => {
                        app.chat.messages.push(message);
                    }
                }
            }
        }
    };

    Ok(())
}
