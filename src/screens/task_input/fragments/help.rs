use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help = Line::from(vec![
        Span::styled("F1/F2", Style::default().fg(Color::Yellow)),
        Span::raw(" switch fragment  "),
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" switch field  "),
        Span::styled("+", Style::default().fg(Color::Yellow)),
        Span::raw(" add heuristic  "),
        Span::styled("Right/Left", Style::default().fg(Color::Yellow)),
        Span::raw(" images/titles  "),
        Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
        Span::raw(" move lists  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" add image (images) / submit (form)  "),
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel"),
    ]);
    let block = Block::default().borders(Borders::ALL).title("Task Input Controls");
    frame.render_widget(Paragraph::new(help).block(block), area);
}
