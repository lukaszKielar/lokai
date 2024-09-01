use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppFocus, AppResult};
use crate::crud::{create_message, get_messages};
use crate::models::Role;

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
                    if let Some(conversation) = app.conversation_list.currently_selected() {
                        // TODO: 1. get text, 2. send text to inference thread, 3. clear input
                        let user_input = app.prompt.text_area.lines().join("\n").trim().to_string();
                        let user_message = create_message(
                            app.sqlite.clone(),
                            Role::User,
                            user_input,
                            conversation.id,
                        )
                        .await?;
                        app.prompt.inference_tx.send(user_message.clone()).await?;
                        app.message_list.items.push(user_message);
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
                app.conversation_list.state.scroll_down_by(1);
                if let Some(current_index) = app.conversation_list.state.selected() {
                    app.message_list.state.select(None);
                    if let Some(item) = app.conversation_list.items.get(current_index) {
                        let messages = get_messages(app.sqlite.clone(), item.id).await?;
                        app.message_list.items = messages;
                    }
                };
            }
            AppFocus::Messages => app.message_list.state.scroll_down_by(1),
            AppFocus::Prompt => {
                app.prompt.text_area.input(key_event);
            }
        },
        KeyCode::Up => match app.current_focus() {
            AppFocus::Conversation => {
                app.conversation_list.state.scroll_up_by(1);
                if let Some(current_index) = app.conversation_list.state.selected() {
                    if let Some(item) = app.conversation_list.items.get(current_index) {
                        let messages = get_messages(app.sqlite.clone(), item.id).await?;
                        app.message_list.items = messages;
                    }
                };
            }
            AppFocus::Messages => app.message_list.state.scroll_up_by(1),
            AppFocus::Prompt => {
                app.prompt.text_area.input(key_event);
            }
        },
        KeyCode::Esc => match app.current_focus() {
            AppFocus::Conversation => {
                app.conversation_list.state.select(None);
                app.message_list.items = vec![];
            }
            AppFocus::Messages => app.message_list.state.select(None),
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
