use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect) {
    let help = Line::from(vec![
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(" new task  "),
        Span::styled("t/d/i", Style::default().fg(Color::Yellow)),
        Span::raw(" fragment  "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(" move (tasks)  "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(" cancel  "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]);
    let block = Block::default().borders(Borders::ALL).title("Controls");
    frame.render_widget(Paragraph::new(help).block(block), area);
}
