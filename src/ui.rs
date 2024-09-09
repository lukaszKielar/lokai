use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Style, Stylize},
    widgets::{Block, BorderType, ListDirection, Padding, Scrollbar, ScrollbarOrientation},
    Frame,
};
use textwrap::Options;

use crate::{
    app::{App, AppFocus},
    models::Role,
};

const FOCUS_BORDER_TYPE: BorderType = BorderType::Double;
const NORMAL_BORDER_TYPE: BorderType = BorderType::Rounded;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    // conversations widget
    let conversations = app
        .conversations
        .as_list_widget(|conversation| conversation.name.trim().to_owned())
        .block(
            Block::bordered()
                .title("CONVERSATIONS")
                .title_alignment(Alignment::Left)
                .border_type(match app.current_focus() {
                    AppFocus::Conversation => FOCUS_BORDER_TYPE,
                    _ => NORMAL_BORDER_TYPE,
                })
                .padding(Padding::new(1, 1, 0, 0)),
        )
        .highlight_style(Style::default().bold())
        .highlight_symbol("👉 ")
        .repeat_highlight_symbol(false)
        .direction(ListDirection::TopToBottom);
    frame.render_stateful_widget(conversations, chunks[0], &mut app.conversations.state);

    // messages widget
    let messages_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(100),
            Constraint::Min(7),
            Constraint::Max(10),
        ])
        .split(chunks[1]);
    let message_padding = Padding::new(1, 1, 0, 0);

    let messages = app
        .chat
        .as_paragraph(
            |message| {
                let width =
                    messages_layout[0].width - (message_padding.left + message_padding.right) * 2;
                let icon = match message.role {
                    Role::Assistant => "🤖",
                    Role::System => "🧰",
                    Role::User => "👤",
                };
                let content =
                    textwrap::wrap(message.content.trim(), Options::new(width as usize)).join("\n");

                format!("{icon} {content}")
            },
            messages_layout[0].height as usize,
        )
        .block(
            Block::bordered()
                .title("CHAT")
                .title_alignment(Alignment::Left)
                .border_type(match app.current_focus() {
                    AppFocus::Messages => FOCUS_BORDER_TYPE,
                    _ => NORMAL_BORDER_TYPE,
                })
                .padding(message_padding),
        )
        .scroll((app.chat.vertical_scroll as u16, 0));
    frame.render_widget(messages, messages_layout[0]);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        messages_layout[0],
        &mut app.chat.vertical_scrollbar_state,
    );

    // TODO: I need to put text to new line when it reaches width of the block
    app.prompt.set_block(
        Block::bordered()
            .title("PROMPT")
            .title_alignment(Alignment::Left)
            .border_type(match app.current_focus() {
                AppFocus::Prompt => FOCUS_BORDER_TYPE,
                _ => NORMAL_BORDER_TYPE,
            }),
    );
    frame.render_widget(&*app.prompt, messages_layout[1]);
}
