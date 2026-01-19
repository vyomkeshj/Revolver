use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;
use crate::screens::FragmentId;
use crate::ui::dashed_border_set;

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect, app: &AppState) {
    let active = app.fragment == FragmentId::MainInput;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let (border_set, border_style) = if active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (ratatui::symbols::border::PLAIN, Style::default())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style)
        .title(Span::styled("Task Input [N]", title_style));
    let value = "Press 'n' to open task input activity.".to_string();
    frame.render_widget(Paragraph::new(value).block(block), area);
}
