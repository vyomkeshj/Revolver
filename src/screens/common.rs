use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw_header(frame: &mut Frame, area: Rect) {
    let text = Text::from(Line::from(Span::styled(
        "Revolver",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    let block = Block::default().borders(Borders::ALL).title("---");
    frame.render_widget(Paragraph::new(text).alignment(Alignment::Center).block(block), area);
}

pub fn selected_list_style() -> Style {
    Style::default()
        .bg(Color::Green)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
}
