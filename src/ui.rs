use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    widgets::{
        Block, BorderType, Borders, Clear, ListDirection, Padding, Paragraph, Scrollbar,
        ScrollbarOrientation,
    },
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
    let dimmed =
        app.new_conversation_popup.is_activated() | app.delete_conversation_popup.is_activated();
    let color = match dimmed {
        true => Color::DarkGray,
        false => Color::White,
    };

    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    // conversations widget
    let conversations = app
        .conversations
        .as_list_widget(|conversation| conversation.name.trim().to_owned())
        .style(color)
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
                let width = messages_layout[0].width
                    - 3
                    - (message_padding.left + message_padding.right) * 2;
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
        .style(color)
        .scroll((app.chat.vertical_scroll as u16, 0));
    frame.render_widget(messages, messages_layout[0]);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .style(color),
        messages_layout[0],
        &mut app.chat.vertical_scrollbar_state,
    );

    // TODO: I need to put text to new line when it reaches width of the block
    // TODO: set prompt to different colors depending on the state of LLM response
    // 1 - green - prompt in a good shape (non empty string)
    // 2 - orange - waiting for LLM's response
    // 3 - red - when user is trying to send empty string or when LLM is still replying
    // 4 - white - initial state (user hasn't started typing yet), also empty prompt
    app.prompt.set_block(
        Block::bordered()
            .title("PROMPT")
            .title_alignment(Alignment::Left)
            .border_type(match app.current_focus() {
                AppFocus::Prompt => FOCUS_BORDER_TYPE,
                _ => NORMAL_BORDER_TYPE,
            })
            .style(color),
    );
    frame.render_widget(&*app.prompt, messages_layout[1]);

    // TODO: dimm other components when popup is active
    if app.new_conversation_popup.is_activated() {
        let (popup_width, popup_height) = (50, 3);
        let (popup_x, popup_y) =
            calculate_coordinates((area.width, area.height), (popup_width, popup_height));
        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
        frame.render_widget(Clear, popup_area);

        app.new_conversation_popup.set_block(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .style(Color::White),
        );
        app.new_conversation_popup
            .set_placeholder_style(Style::default());
        app.new_conversation_popup
            .set_placeholder_text("Give a name to your new conversation");
        frame.render_widget(&*app.new_conversation_popup, popup_area);
    }

    if app.delete_conversation_popup.is_activated() {
        let popup_message = "Would you like to delete conversation? <Y/n>";
        let (popup_width, popup_height) = (50, 3);
        let (popup_x, popup_y) =
            calculate_coordinates((area.width, area.height), (popup_width, popup_height));

        let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
        frame.render_widget(Clear, popup_area);

        let paragraph = Paragraph::new(popup_message)
            .centered()
            .block(Block::bordered().border_type(BorderType::Rounded))
            .style(Color::White);
        frame.render_widget(paragraph, popup_area);
    }
}

fn calculate_coordinates(area_size: (u16, u16), elem_size: (u16, u16)) -> (u16, u16) {
    (
        area_size.0 / 2 - elem_size.0 / 2,
        area_size.1 / 2 - elem_size.1 / 2,
    )
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case((50, 30), (15, 5), (18,13))]
    #[case((50,30), (10, 2), (20,14))]
    #[case((40,20), (15,5), (13,8))]
    #[case((40,10), (10,2), (15,4))]
    #[case((30, 30), (15,2), (8,14))]
    #[case((30, 30), (10,29), (10,1))]
    #[case((25, 5), (15,3), (5,1))]
    #[case((25, 2), (10,1), (7,1))]
    #[case((25, 2), (10,2), (7,0))]
    fn test_calculate_size(
        #[case] area_size: (u16, u16),
        #[case] elem_size: (u16, u16),
        #[case] expected: (u16, u16),
    ) {
        // when
        let output = calculate_coordinates(area_size, elem_size);

        // then
        assert_eq!(output, expected);
    }
}
