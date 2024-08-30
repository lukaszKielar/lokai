use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::{Block, BorderType, List, ListDirection, ListItem},
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
        .map(|elem| elem.name.to_owned().into())
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
        .highlight_symbol(">> ")
        .repeat_highlight_symbol(true)
        .direction(ListDirection::TopToBottom);
    frame.render_stateful_widget(conversations, chunks[0], &mut app.conversation_list.state);

    // messages widget
    let messages_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(7),
            Constraint::Max(10),
        ])
        .split(chunks[1]);
    let items = app
        .message_list
        .items
        .iter()
        .map(|elem| elem.content.to_owned().into())
        .collect::<Vec<ListItem>>();
    let messages = List::new(items).block(
        Block::bordered()
            .title("CHAT")
            .title_alignment(Alignment::Left)
            .border_type(match app.current_focus() {
                AppFocus::Messages => FOCUS_BORDER_TYPE,
                _ => NORMAL_BORDER_TYPE,
            }),
    );
    frame.render_stateful_widget(messages, messages_layout[0], &mut app.message_list.state);

    // TODO: I need to put text to new line when it reaches width of the block
    app.prompt.text_area.set_block(
        Block::bordered()
            .title("PROMPT")
            .title_alignment(Alignment::Left)
            .border_type(match app.current_focus() {
                AppFocus::Prompt => FOCUS_BORDER_TYPE,
                _ => NORMAL_BORDER_TYPE,
            }),
    );
    frame.render_widget(&app.prompt.text_area, messages_layout[1]);
}
