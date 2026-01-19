use ratatui::layout::Constraint;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Row, Table, TableState};
use ratatui::Frame;

use crate::app::AppState;
use crate::screens::common::selected_list_style;
use crate::screens::{FragmentId};
use crate::ui::{dashed_border_set, truncate, format_status};

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect, app: &AppState) {
    let tasks = app.tasks_in_order();
    let header = Row::new(vec![
        "ID", "Name", "Status", "Progress", "Iter", "V", "X",
    ])
    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

    let rows = tasks.iter().map(|task| {
        Row::new(vec![
            task.id.to_string(),
            truncate(&task.name, 16),
            format_status(&task.status),
            format!("{:.0}%", task.progress * 100.0),
            format!("{}/{}", task.iteration, task.max_iters),
            task.verified.len().to_string(),
            task.discarded.len().to_string(),
        ])
    });

    let active = app.fragment == FragmentId::MainTasks;
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

    let table = Table::new(rows, [
        Constraint::Length(4),
        Constraint::Length(16),
        Constraint::Length(11),
        Constraint::Length(9),
        Constraint::Length(8),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .header(header)
    .highlight_style(selected_list_style())
    .highlight_symbol(" ")
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border_set)
            .border_style(border_style)
            .title(Span::styled("Tasks [T]", title_style)),
    );

    let mut state = TableState::default();
    if !tasks.is_empty() {
        state.select(Some(app.selected));
    }
    frame.render_stateful_widget(table, area, &mut state);
}
