use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, TableState};
use ratatui::Frame;

use crate::app::{Activity, AppState, DraftField, Fragment, HeuristicsFocus};
use crate::task::{Hypothesis, TaskPhase, TaskStatus};

pub fn draw(frame: &mut Frame, app: &AppState) {
    if app.activity == Activity::TaskInput {
        draw_task_input_screen(frame, app);
        return;
    }

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.size());

    draw_header(frame, root[0]);
    draw_main(frame, root[1], app);
    draw_input(frame, root[2], app);
    draw_help(frame, root[3], app);
}

pub fn draw_splash(frame: &mut Frame) {
    let block = Block::default().borders(Borders::ALL).title("Revolver");
    let text = Text::from(vec![
        Line::from(" Revolver"),
    ]);
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    let area = frame.size();
    let desired_height = 3u16;
    let pad = area.height.saturating_sub(desired_height) / 2;
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(pad),
            Constraint::Length(desired_height),
            Constraint::Min(0),
        ])
        .split(area);
    frame.render_widget(paragraph, layout[1]);
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let text = Text::from(Line::from(Span::styled(
        "Revolver",
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    let block = Block::default().borders(Borders::ALL).title("Header");
    frame.render_widget(
        Paragraph::new(text).alignment(ratatui::layout::Alignment::Center).block(block),
        area,
    );
}

fn draw_main(frame: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    draw_task_list(frame, chunks[0], app);
    draw_task_detail(frame, chunks[1], app);
}

fn draw_task_list(frame: &mut Frame, area: Rect, app: &AppState) {
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

    let active = app.fragment == Fragment::MainTasks;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let highlight_style = selected_list_style();
    let (border_set, border_style) = if active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (border::PLAIN, Style::default())
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
    .highlight_style(highlight_style)
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

fn draw_task_detail(frame: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Min(6)])
        .split(area);

    draw_task_summary(frame, chunks[0], app);
    draw_hypothesis_lists(frame, chunks[1], app);
}

fn draw_task_summary(frame: &mut Frame, area: Rect, app: &AppState) {
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
        if let TaskStatus::Failed(error) = &task.status {
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

    let active = app.fragment == Fragment::MainDetail;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let (border_set, border_style) = if active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (border::PLAIN, Style::default())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style)
        .title(Span::styled("Task Detail [D]", title_style));
    frame.render_widget(Paragraph::new(content).block(block), area);
}

fn draw_hypothesis_lists(frame: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let (verified, discarded) = app
        .selected_task()
        .map(|task| (task.verified, task.discarded))
        .unwrap_or_default();

    let verified_items = list_items(&verified);
    let discarded_items = list_items(&discarded);

    let verified_list = List::new(verified_items)
        .block(Block::default().borders(Borders::ALL).title("Verified"));
    let discarded_list = List::new(discarded_items)
        .block(Block::default().borders(Borders::ALL).title("Discarded"));

    frame.render_widget(verified_list, chunks[0]);
    frame.render_widget(discarded_list, chunks[1]);
}

fn draw_input(frame: &mut Frame, area: Rect, app: &AppState) {
    let active = app.fragment == Fragment::MainInput;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let title = "Task Input [N]";
    let (border_set, border_style) = if active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (border::PLAIN, Style::default())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style)
        .title(Span::styled(title, title_style));
    let value = "Press 'n' to open task input activity.".to_string();
    frame.render_widget(Paragraph::new(value).block(block), area);
}

fn draw_help(frame: &mut Frame, area: Rect, _app: &AppState) {
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

fn draw_task_input_screen(frame: &mut Frame, app: &AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(frame.size());

    draw_header(frame, root[0]);

    if let Some(cursor) = draw_task_description_fragment(frame, root[1], app) {
        if app.cursor_visible {
            frame.set_cursor(cursor.0, cursor.1);
        }
    }
    draw_hypothesis_fragment(frame, root[2], app);
    draw_task_input_help(frame, root[3], app);
}

fn draw_task_description_fragment(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let is_active = app.fragment == Fragment::TaskDescription;
    let title_style = if is_active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let (border_set, border_style) = if is_active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (border::PLAIN, Style::default())
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_set(border_set)
        .border_style(border_style)
        .title(Span::styled("Task Description [F1]", title_style));
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(32),
            Constraint::Percentage(32),
            Constraint::Percentage(36),
        ])
        .split(inner);

    let name_cursor = draw_name_box(frame, columns[0], app);
    let dataset_cursor = draw_dataset_box(frame, columns[1], app);
    let heuristics_cursor = draw_heuristics_box(frame, columns[2], app);
    name_cursor.or(dataset_cursor).or(heuristics_cursor)
}

fn draw_name_box(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active = app.fragment == Fragment::TaskDescription && app.draft.field == DraftField::Name;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let value = if app.draft.field == DraftField::Name {
        app.input.clone()
    } else {
        app.draft.name.clone()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Name", title_style));
    frame.render_widget(Paragraph::new(value).block(block), area);
    if active {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1;
        return Some((x, y));
    }
    None
}

fn draw_dataset_box(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active =
        app.fragment == Fragment::TaskDescription && app.draft.field == DraftField::DatasetFolder;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let value = if app.draft.field == DraftField::DatasetFolder {
        app.input.clone()
    } else {
        app.draft.dataset_folder.clone()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Dataset Folder", title_style));
    frame.render_widget(Paragraph::new(value).block(block), area);
    if active {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1;
        return Some((x, y));
    }
    None
}

fn draw_heuristics_box(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active =
        app.fragment == Fragment::TaskDescription && app.draft.field == DraftField::Heuristics;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Heuristics [+ to add]", title_style));
    frame.render_widget(block.clone(), area);

    let inner = block.inner(area);
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    let title_cursor = draw_heuristic_titles(frame, columns[0], app);
    let image_cursor = draw_heuristic_images(frame, columns[1], app);
    title_cursor.or(image_cursor)
}

fn draw_heuristic_titles(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active = app.fragment == Fragment::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Titles;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let mut items = Vec::new();
    if app.draft.heuristics.is_empty() {
        items.push(ListItem::new(Span::styled(
            "[Empty List]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, heuristic) in app.draft.heuristics.iter().enumerate() {
            let selected = idx == app.draft.selected_heuristic;
            let style = if active && selected {
                selected_list_style()
            } else {
                Style::default()
            };
            items.push(ListItem::new(Span::styled(
                truncate(&heuristic.title, 24),
                style,
            )));
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Titles", title_style));
    frame.render_widget(List::new(items).block(block), area);
    if active && !app.draft.heuristics.is_empty() {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1 + app.draft.selected_heuristic as u16;
        return Some((x, y));
    }
    None
}

fn draw_heuristic_images(frame: &mut Frame, area: Rect, app: &AppState) -> Option<(u16, u16)> {
    let active = app.fragment == Fragment::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Images;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let images = app
        .draft
        .heuristics
        .get(app.draft.selected_heuristic)
        .map(|h| h.images.as_slice())
        .unwrap_or(&[]);
    let mut items = Vec::new();
    if images.is_empty() {
        items.push(ListItem::new(Span::styled(
            "[Empty List]",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, image) in images.iter().enumerate() {
            let selected = idx == app.draft.selected_image;
            let style = if active && selected {
                selected_list_style()
            } else {
                Style::default()
            };
            items.push(ListItem::new(Span::styled(
                truncate(image, 24),
                style,
            )));
        }
    }
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Images (Right arrow)", title_style));
    frame.render_widget(List::new(items).block(block), area);
    if active && !images.is_empty() {
        let x = area.x + 1 + app.cursor_pos.min(area.width.saturating_sub(2) as usize) as u16;
        let y = area.y + 1 + app.draft.selected_image as u16;
        return Some((x, y));
    }
    None
}

fn draw_hypothesis_fragment(frame: &mut Frame, area: Rect, app: &AppState) {
    let active = app.fragment == Fragment::TaskHypotheses;
    let title_style = if active {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let (border_set, border_style) = if active {
        (dashed_border_set(), Style::default().fg(Color::Green))
    } else {
        (border::PLAIN, Style::default())
    };
    let hypothesis_items = app
        .draft
        .hypotheses
        .iter()
        .enumerate()
        .map(|(idx, h)| {
            let selected = idx == app.draft.selected_hypothesis;
            let style = if active && selected {
                selected_list_style()
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(truncate(&h.title, 60), style))
        })
        .collect::<Vec<_>>();

    let hypothesis_list = List::new(hypothesis_items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_set(border_set)
            .border_style(border_style)
            .title(Span::styled("Hypotheses [F2]", title_style)),
    );

    frame.render_widget(hypothesis_list, area);
}

fn draw_task_input_help(frame: &mut Frame, area: Rect, _app: &AppState) {
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

fn list_items(items: &[Hypothesis]) -> Vec<ListItem<'_>> {
    if items.is_empty() {
        return vec![ListItem::new(Span::raw("none yet"))];
    }
    items
        .iter()
        .map(|item| {
            ListItem::new(Line::from(format!(
                "{:.2}  {}",
                item.score,
                truncate(&item.description, 42)
            )))
        })
        .collect()
}

fn format_status(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Pending => "Pending".to_string(),
        TaskStatus::Running => "Running".to_string(),
        TaskStatus::Cancelled => "Cancelled".to_string(),
        TaskStatus::Done => "Done".to_string(),
        TaskStatus::Failed(_) => "Failed".to_string(),
    }
}

fn format_phase(phase: &TaskPhase) -> String {
    match phase {
        TaskPhase::Defining => "Defining".to_string(),
        TaskPhase::GeneratingHypotheses => "Generating".to_string(),
        TaskPhase::EvaluatingHypotheses => "Evaluating".to_string(),
        TaskPhase::Reducing => "Reducing".to_string(),
        TaskPhase::Synthesizing => "Synthesizing".to_string(),
        TaskPhase::Testing => "Testing".to_string(),
        TaskPhase::Reporting => "Reporting".to_string(),
        TaskPhase::Finished => "Finished".to_string(),
    }
}

fn truncate(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    if max <= 3 {
        return value.chars().take(max).collect();
    }
    let mut out = value.chars().take(max - 3).collect::<String>();
    out.push_str("...");
    out
}

fn selected_list_style() -> Style {
    Style::default()
        .bg(Color::Green)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD)
}

fn dashed_border_set() -> border::Set {
    border::Set {
        top_left: "┌",
        top_right: "┐",
        bottom_left: "└",
        bottom_right: "┘",
        vertical_left: "┆",
        vertical_right: "┆",
        horizontal_top: "┄",
        horizontal_bottom: "┄",
    }
}
