use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::AppState;
use crate::screens::FragmentId;
use crate::ui::{format_phase, format_status, truncate, dashed_border_set};

pub fn draw(frame: &mut Frame, area: ratatui::layout::Rect, app: &AppState) {
    let content = if let Some(task) = app.selected_task() {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                task.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from(format!(
                "Status: {} | Phase: {} | Progress: {:.0}%",
                format_status(&task.status),
                format_phase(&task.phase),
                task.progress * 100.0
            )),
        ];
        if let crate::task::TaskStatus::Failed(error) = &task.status {
            lines.push(Line::from(format!("Error: {}", truncate(error, 48))));
        }
        lines.extend(vec![
            Line::from(format!(
                "Iter: {}/{} | Best: {:.2} | Last: {:.2}",
                task.iteration, task.max_iters, task.best_score, task.last_score
            )),
            Line::from(format!(
                "Dataset: {} images | edge {:.2} | min_blob {} | contrast {:.2}",
                task.dataset_size,
                task.heuristics.edge_threshold,
                task.heuristics.min_blob_area,
                task.heuristics.contrast_boost
            )),
            Line::from(format!(
                "Report: {}",
                task.report_path
                    .clone()
                    .unwrap_or_else(|| "pending".to_string())
            )),
            Line::from(format!(
                "Recent: {}",
                app.recent_logs()
                    .last()
                    .cloned()
                    .unwrap_or_else(|| "none".to_string())
            )),
        ]);
        Text::from(lines)
    } else {
        Text::from("No tasks yet. Press 'n' to add a task.")
    };

    let active = app.fragment == FragmentId::MainDetail;
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
        .title(Span::styled("Task Detail [D]", title_style));
    frame.render_widget(Paragraph::new(content).block(block), area);
}
