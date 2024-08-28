use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppFocus, AppResult};

pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Ctrl + c -> exit
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Down => match app.current_focus() {
            AppFocus::Conversation => app.conversation_list.state.scroll_down_by(1),
            AppFocus::Messages => todo!(),
            AppFocus::Prompt => todo!(),
        },
        KeyCode::Up => match app.current_focus() {
            AppFocus::Conversation => app.conversation_list.state.scroll_up_by(1),
            AppFocus::Messages => todo!(),
            AppFocus::Prompt => todo!(),
        },
        KeyCode::Esc => app.conversation_list.state.select(None),
        KeyCode::Tab => app.next_focus(),
        KeyCode::BackTab => app.previous_focus(),
        _ => {}
    }
    Ok(())
}
