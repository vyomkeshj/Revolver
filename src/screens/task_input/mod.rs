pub mod fragments;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::app::{AppState, DraftField, HeuristicsFocus};
use crate::scheduler::SchedulerCommand;
use crate::screens::{FragmentId, KeyBinding, Screen};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Close,
    SwitchField,
    FocusDescription,
    FocusHypotheses,
    CursorLeft,
    CursorRight,
    MoveUp,
    MoveDown,
    AddHeuristic,
    Submit,
    Backspace,
}

const KEY_BINDINGS: &[KeyBinding<Action>] = &[
    KeyBinding {
        key: KeyCode::Esc,
        action: Action::Close,
    },
    KeyBinding {
        key: KeyCode::Tab,
        action: Action::SwitchField,
    },
    KeyBinding {
        key: KeyCode::F(1),
        action: Action::FocusDescription,
    },
    KeyBinding {
        key: KeyCode::F(2),
        action: Action::FocusHypotheses,
    },
    KeyBinding {
        key: KeyCode::Left,
        action: Action::CursorLeft,
    },
    KeyBinding {
        key: KeyCode::Right,
        action: Action::CursorRight,
    },
    KeyBinding {
        key: KeyCode::Up,
        action: Action::MoveUp,
    },
    KeyBinding {
        key: KeyCode::Down,
        action: Action::MoveDown,
    },
    KeyBinding {
        key: KeyCode::Char('+'),
        action: Action::AddHeuristic,
    },
    KeyBinding {
        key: KeyCode::Enter,
        action: Action::Submit,
    },
    KeyBinding {
        key: KeyCode::Backspace,
        action: Action::Backspace,
    },
];

pub struct TaskInputScreen;

impl Screen for TaskInputScreen {
    fn draw(&self, frame: &mut Frame, app: &AppState) {
        draw(frame, app);
    }

    fn handle_key(
        &self,
        key: KeyCode,
        app: &mut AppState,
        cmd_tx: &mpsc::Sender<SchedulerCommand>,
    ) -> std::io::Result<bool> {
        handle_key(key, app, cmd_tx)
    }
}

pub fn handle_key(
    key: KeyCode,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> std::io::Result<bool> {
    if let Some(binding) = KEY_BINDINGS.iter().find(|b| b.key == key) {
        return handle_action(binding.action, app, cmd_tx);
    }
    if let KeyCode::Char(ch) = key {
        return handle_char(ch, app);
    }
    Ok(false)
}

fn handle_action(
    action: Action,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> std::io::Result<bool> {
    match action {
        Action::Close => app.close_task_input(),
        Action::SwitchField => {
            if app.fragment == FragmentId::TaskDescription {
                app.commit_draft_field();
                app.draft.field = match app.draft.field {
                    DraftField::Name => DraftField::DatasetFolder,
                    DraftField::DatasetFolder => DraftField::Heuristics,
                    DraftField::Heuristics => DraftField::Name,
                };
                app.load_draft_field();
            }
        }
        Action::FocusDescription => app.set_fragment(FragmentId::TaskDescription),
        Action::FocusHypotheses => app.set_fragment(FragmentId::TaskHypotheses),
        Action::CursorLeft => move_cursor_left(app),
        Action::CursorRight => move_cursor_right(app),
        Action::MoveUp => move_selection_up(app),
        Action::MoveDown => move_selection_down(app),
        Action::AddHeuristic => {
            if app.fragment == FragmentId::TaskDescription
                && app.draft.field == DraftField::Heuristics
            {
                if app.draft.heuristics_focus == HeuristicsFocus::Images {
                    add_image(app);
                } else {
                    let title = format!("Heuristic {}", app.draft.heuristics.len() + 1);
                    app.add_heuristic(title);
                }
            }
        }
        Action::Submit => {
            if app.fragment == FragmentId::TaskDescription
                && app.draft.field == DraftField::Heuristics
                && app.draft.heuristics_focus == HeuristicsFocus::Images
            {
                add_image(app);
                return Ok(false);
            }
            app.commit_draft_field();
            let name = app.draft.name.trim().to_string();
            if !name.is_empty() {
                let _ = cmd_tx.try_send(SchedulerCommand::AddTask { name });
            }
            app.reset_draft();
            app.close_task_input();
        }
        Action::Backspace => handle_backspace(app),
    }
    Ok(false)
}

fn move_cursor_left(app: &mut AppState) {
    if app.fragment == FragmentId::TaskDescription && app.draft.field == DraftField::Heuristics {
        if app.draft.heuristics_focus == HeuristicsFocus::Images {
            if app.cursor_pos == 0 {
                app.draft.heuristics_focus = HeuristicsFocus::Titles;
                app.cursor_pos = app
                    .draft
                    .heuristics
                    .get(app.draft.selected_heuristic)
                    .map(|h| h.title.len())
                    .unwrap_or(0);
            } else if let Some(image) = app
                .draft
                .heuristics
                .get(app.draft.selected_heuristic)
                .and_then(|h| h.images.get(app.draft.selected_image))
            {
                app.cursor_pos = app.cursor_pos.saturating_sub(1).min(image.len());
            }
        } else if let Some(title) = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .map(|h| h.title.as_str())
        {
            app.cursor_pos = app.cursor_pos.saturating_sub(1).min(title.len());
        }
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field != DraftField::Heuristics
    {
        app.cursor_pos = app.cursor_pos.saturating_sub(1).min(app.input.len());
    }
}

fn move_cursor_right(app: &mut AppState) {
    if app.fragment == FragmentId::TaskDescription && app.draft.field == DraftField::Heuristics {
        if app.draft.heuristics_focus == HeuristicsFocus::Titles {
            if let Some(title) = app
                .draft
                .heuristics
                .get(app.draft.selected_heuristic)
                .map(|h| h.title.as_str())
            {
                if app.cursor_pos >= title.len() {
                    app.draft.heuristics_focus = HeuristicsFocus::Images;
                    app.cursor_pos = app
                        .draft
                        .heuristics
                        .get(app.draft.selected_heuristic)
                        .and_then(|h| h.images.get(app.draft.selected_image))
                        .map(|s| s.len())
                        .unwrap_or(0);
                } else {
                    app.cursor_pos = (app.cursor_pos + 1).min(title.len());
                }
            }
        } else if let Some(image) = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .and_then(|h| h.images.get(app.draft.selected_image))
        {
            app.cursor_pos = (app.cursor_pos + 1).min(image.len());
        }
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field != DraftField::Heuristics
    {
        app.cursor_pos = (app.cursor_pos + 1).min(app.input.len());
    }
}

fn move_selection_up(app: &mut AppState) {
    if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Titles
        && app.draft.selected_heuristic > 0
    {
        app.draft.selected_heuristic -= 1;
        app.draft.selected_image = 0;
        app.cursor_pos = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .map(|h| h.title.len())
            .unwrap_or(0);
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Images
        && app.draft.selected_image > 0
    {
        app.draft.selected_image -= 1;
        app.cursor_pos = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .and_then(|h| h.images.get(app.draft.selected_image))
            .map(|s| s.len())
            .unwrap_or(0);
    } else if app.fragment == FragmentId::TaskHypotheses
        && app.draft.selected_hypothesis > 0
    {
        app.draft.selected_hypothesis -= 1;
    }
}

fn move_selection_down(app: &mut AppState) {
    if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Titles
    {
        let max = app.draft.heuristics.len().saturating_sub(1);
        app.draft.selected_heuristic = (app.draft.selected_heuristic + 1).min(max);
        app.draft.selected_image = 0;
        app.cursor_pos = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .map(|h| h.title.len())
            .unwrap_or(0);
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Images
    {
        let count = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .map(|h| h.images.len())
            .unwrap_or(0);
        let max = count.saturating_sub(1);
        app.draft.selected_image = (app.draft.selected_image + 1).min(max);
        app.cursor_pos = app
            .draft
            .heuristics
            .get(app.draft.selected_heuristic)
            .and_then(|h| h.images.get(app.draft.selected_image))
            .map(|s| s.len())
            .unwrap_or(0);
    } else if app.fragment == FragmentId::TaskHypotheses {
        let max = app.draft.hypotheses.len().saturating_sub(1);
        app.draft.selected_hypothesis =
            (app.draft.selected_hypothesis + 1).min(max);
    }
}

fn add_image(app: &mut AppState) {
    if let Some(heuristic) = app
        .draft
        .heuristics
        .get_mut(app.draft.selected_heuristic)
    {
        heuristic.images.push(String::new());
        app.draft.selected_image = heuristic.images.len().saturating_sub(1);
        app.cursor_pos = 0;
    }
}

fn handle_backspace(app: &mut AppState) {
    if app.fragment == FragmentId::TaskDescription && app.draft.field != DraftField::Heuristics {
        if app.cursor_pos > 0 {
            app.cursor_pos -= 1;
            app.input.remove(app.cursor_pos);
        }
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Titles
    {
        if let Some(heuristic) = app.draft.heuristics.get_mut(app.draft.selected_heuristic) {
            if app.cursor_pos > 0 && app.cursor_pos <= heuristic.title.len() {
                app.cursor_pos -= 1;
                heuristic.title.remove(app.cursor_pos);
            }
        }
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Images
    {
        if let Some(heuristic) = app.draft.heuristics.get_mut(app.draft.selected_heuristic) {
            if let Some(image) = heuristic.images.get_mut(app.draft.selected_image) {
                if app.cursor_pos > 0 && app.cursor_pos <= image.len() {
                    app.cursor_pos -= 1;
                    image.remove(app.cursor_pos);
                }
            }
        }
    }
}

fn handle_char(ch: char, app: &mut AppState) -> std::io::Result<bool> {
    if app.fragment == FragmentId::TaskDescription && app.draft.field != DraftField::Heuristics {
        if app.cursor_pos <= app.input.len() {
            app.input.insert(app.cursor_pos, ch);
            app.cursor_pos += 1;
        }
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Titles
    {
        if let Some(heuristic) = app.draft.heuristics.get_mut(app.draft.selected_heuristic) {
            if app.cursor_pos <= heuristic.title.len() {
                heuristic.title.insert(app.cursor_pos, ch);
                app.cursor_pos += 1;
            }
        }
    } else if app.fragment == FragmentId::TaskDescription
        && app.draft.field == DraftField::Heuristics
        && app.draft.heuristics_focus == HeuristicsFocus::Images
    {
        if let Some(heuristic) = app.draft.heuristics.get_mut(app.draft.selected_heuristic) {
            if heuristic.images.is_empty() {
                heuristic.images.push(String::new());
                app.draft.selected_image = 0;
                app.cursor_pos = 0;
            }
            if let Some(image) = heuristic.images.get_mut(app.draft.selected_image) {
                if app.cursor_pos <= image.len() {
                    image.insert(app.cursor_pos, ch);
                    app.cursor_pos += 1;
                }
            }
        }
    }
    Ok(false)
}

pub fn draw(frame: &mut Frame, app: &AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(12),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(frame.size());

    fragments::header::draw(frame, root[0]);

    if let Some(cursor) = fragments::description::draw(frame, root[1], app) {
        if app.cursor_visible {
            frame.set_cursor(cursor.0, cursor.1);
        }
    }
    fragments::hypotheses::draw(frame, root[2], app);
    fragments::help::draw(frame, root[3]);
}
