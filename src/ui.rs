use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    widgets::{Block, BorderType, Paragraph},
    Frame,
};

use crate::app::App;

pub fn render(_app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(area);

    frame.render_widget(
        Paragraph::new("AAA")
            .block(
                Block::bordered()
                    .title("Conversations")
                    .title_alignment(Alignment::Left)
                    .border_type(BorderType::Rounded),
            )
            .centered(),
        layout[0],
    );
    frame.render_widget(
        Paragraph::new("BBB")
            .alignment(Alignment::Left)
            .block(
                Block::bordered()
                    .title("Messages")
                    .title_alignment(Alignment::Left)
                    .border_type(BorderType::Rounded),
            )
            .centered(),
        layout[1],
    )
}
