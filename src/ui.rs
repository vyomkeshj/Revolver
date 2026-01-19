use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::task::{TaskPhase, TaskStatus};

pub fn draw_splash(frame: &mut Frame) {
    let block = Block::default().borders(Borders::ALL).title("TheLastMachine");
    let text = Text::from(vec![Line::from(" TheLastMachine")]);
    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
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

pub fn format_status(status: &TaskStatus) -> String {
    match status {
        TaskStatus::Pending => "Pending".to_string(),
        TaskStatus::Running => "Running".to_string(),
        TaskStatus::Cancelled => "Cancelled".to_string(),
        TaskStatus::Done => "Done".to_string(),
        TaskStatus::Failed(_) => "Failed".to_string(),
    }
}

pub fn format_phase(phase: &TaskPhase) -> String {
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

pub fn truncate(value: &str, max: usize) -> String {
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

pub fn dashed_border_set() -> border::Set {
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
