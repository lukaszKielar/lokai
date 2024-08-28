use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListDirection, ListItem, Paragraph},
    Frame,
};

const FOCUS_BORDER_TYPE: BorderType = BorderType::Double;
const NORMAL_BORDER_TYPE: BorderType = BorderType::Rounded;

use crate::app::{App, AppFocus};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    // conversations widget
    let items = app
        .conversation_list
        .items
        .iter()
        .map(|elem| ListItem::new(elem.as_str()))
        .collect::<Vec<ListItem>>();
    let conversations = List::new(items)
        .block(
            Block::bordered()
                .title("CONVERSATIONS")
                .title_alignment(Alignment::Left)
                .border_type(match app.current_focus() {
                    AppFocus::Conversation => FOCUS_BORDER_TYPE,
                    _ => NORMAL_BORDER_TYPE,
                }),
        )
        .highlight_style(Style::default().bold())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);
    frame.render_stateful_widget(conversations, chunks[0], &mut app.conversation_list.state);

    // messages widget
    let messages_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(5),
            Constraint::Max(5),
        ])
        .split(chunks[1]);
    let messages = Paragraph::new("CHAT")
        .block(
            Block::bordered()
                .title("CHAT")
                .title_alignment(Alignment::Left)
                .border_type(match app.current_focus() {
                    AppFocus::Messages => FOCUS_BORDER_TYPE,
                    _ => NORMAL_BORDER_TYPE,
                }),
        )
        .centered();
    frame.render_widget(messages, messages_layout[0]);

    let prompt = Paragraph::new("PROMPT")
        .block(
            Block::bordered()
                .title("PROMPT")
                .title_alignment(Alignment::Left)
                .border_type(match app.current_focus() {
                    AppFocus::Prompt => FOCUS_BORDER_TYPE,
                    _ => NORMAL_BORDER_TYPE,
                }),
        )
        .centered();
    frame.render_widget(prompt, messages_layout[1]);
}
