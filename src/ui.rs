use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, TableState};
use ratatui::Frame;

use crate::app::{AppState, DraftField, DraftFocus, FocusSection, Screen};
use crate::task::{Hypothesis, TaskPhase, TaskStatus};

pub fn draw(frame: &mut Frame, app: &AppState) {
    if app.screen == Screen::TaskInput {
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
    let block = Block::default().borders(Borders::ALL).title("TheLastMachine");
    let text = Text::from(vec![
        Line::from(" TheLastMachine"),
    ]);
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);
    frame.render_widget(paragraph, frame.size());
}

fn draw_header(frame: &mut Frame, area: Rect) {
    let text = Text::from(Line::from(Span::styled(
        "TheLastMachine",
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

    let title_style = if app.focus == FocusSection::Tasks {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let highlight_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD | Modifier::REVERSED);
    let border_set = if app.focus == FocusSection::Tasks && !tasks.is_empty() {
        dashed_border_set()
    } else {
        border::PLAIN
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

    let title_style = if app.focus == FocusSection::Detail {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let block = Block::default()
        .borders(Borders::ALL)
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
    let title_style = if app.focus == FocusSection::Input {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let title = if app.input_mode {
        "New Task (Enter to submit, Esc to cancel)"
    } else {
        "Task Input [N]"
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, title_style));
    let value = if app.input_mode {
        app.input.clone()
    } else {
        "Press 'n' to create a task.".to_string()
    };
    frame.render_widget(Paragraph::new(value).block(block), area);
}

fn draw_help(frame: &mut Frame, area: Rect, _app: &AppState) {
    let help = Line::from(vec![
        Span::styled("n", Style::default().fg(Color::Yellow)),
        Span::raw(" new task  "),
        Span::styled("t/d", Style::default().fg(Color::Yellow)),
        Span::raw(" focus  "),
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
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(3)])
        .split(frame.size());

    draw_header(frame, root[0]);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(root[1]);

    draw_task_form(frame, main[0], app);
    draw_hypothesis_panel(frame, main[1], app);
    draw_task_input_help(frame, root[2], app);
}

fn draw_task_form(frame: &mut Frame, area: Rect, app: &AppState) {
    let is_field_focus = app.draft.focus == DraftFocus::Fields;
    let name_style = if is_field_focus && app.draft.field == DraftField::Name {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };
    let dataset_style = if is_field_focus && app.draft.field == DraftField::DatasetFolder {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else {
        Style::default()
    };

    let lines = vec![
        Line::from(Span::styled("Task Definition", Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::raw("Name: "),
            Span::styled(
                if app.draft.field == DraftField::Name {
                    app.input.clone()
                } else {
                    app.draft.name.clone()
                },
                name_style,
            ),
        ]),
        Line::from(vec![
            Span::raw("Dataset folder: "),
            Span::styled(
                if app.draft.field == DraftField::DatasetFolder {
                    app.input.clone()
                } else {
                    app.draft.dataset_folder.clone()
                },
                dataset_style,
            ),
        ]),
        Line::from(""),
        Line::from("Heuristics (mock):"),
        Line::from(" - edge_threshold: 0.42"),
        Line::from(" - min_blob_area: 120"),
        Line::from(" - contrast_boost: 1.25"),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Task Input [Tab to switch]");
    frame.render_widget(Paragraph::new(Text::from(lines)).block(block), area);
}

fn draw_hypothesis_panel(frame: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let hypothesis_items = app
        .draft
        .hypotheses
        .iter()
        .enumerate()
        .map(|(idx, h)| {
            let selected = idx == app.draft.selected_hypothesis;
            let style = if app.draft.focus == DraftFocus::Hypotheses && selected {
                Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            ListItem::new(Span::styled(truncate(&h.title, 36), style))
        })
        .collect::<Vec<_>>();

    let hypothesis_list = List::new(hypothesis_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Hypotheses [H]"),
    );

    frame.render_widget(hypothesis_list, chunks[0]);

    let images = app
        .draft
        .hypotheses
        .get(app.draft.selected_hypothesis)
        .map(|h| h.images.as_slice())
        .unwrap_or(&[]);

    let image_items = if images.is_empty() {
        vec![ListItem::new("none")]
    } else {
        images
            .iter()
            .enumerate()
            .map(|(idx, img)| {
                let selected = idx == app.draft.selected_image;
                let style = if app.draft.focus == DraftFocus::Images && selected {
                    Style::default().fg(Color::Green).add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(Span::styled(img.clone(), style))
            })
            .collect()
    };

    let image_list = List::new(image_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Images [I]"),
    );
    frame.render_widget(image_list, chunks[1]);
}

fn draw_task_input_help(frame: &mut Frame, area: Rect, _app: &AppState) {
    let help = Line::from(vec![
        Span::styled("Tab", Style::default().fg(Color::Yellow)),
        Span::raw(" switch field  "),
        Span::styled("Up/Down", Style::default().fg(Color::Yellow)),
        Span::raw(" move lists  "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" submit  "),
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
