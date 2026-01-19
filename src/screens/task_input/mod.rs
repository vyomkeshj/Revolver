pub mod fragments;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::app::AppState;
use crate::scheduler::SchedulerCommand;
use crate::screens::{KeyBinding, Screen};

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
        app.enqueue_event(crate::app::AppEvent::DraftInsertChar(ch));
        return Ok(false);
    }
    Ok(false)
}

fn handle_action(
    action: Action,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> std::io::Result<bool> {
    let _ = cmd_tx;
    match action {
        Action::Close => app.enqueue_event(crate::app::AppEvent::CloseTaskInput),
        Action::SwitchField => app.enqueue_event(crate::app::AppEvent::DraftSwitchField),
        Action::FocusDescription => app.enqueue_event(crate::app::AppEvent::DraftFocusDescription),
        Action::FocusHypotheses => app.enqueue_event(crate::app::AppEvent::DraftFocusHypotheses),
        Action::CursorLeft => app.enqueue_event(crate::app::AppEvent::DraftCursorLeft),
        Action::CursorRight => app.enqueue_event(crate::app::AppEvent::DraftCursorRight),
        Action::MoveUp => app.enqueue_event(crate::app::AppEvent::DraftMoveUp),
        Action::MoveDown => app.enqueue_event(crate::app::AppEvent::DraftMoveDown),
        Action::AddHeuristic => app.enqueue_event(crate::app::AppEvent::DraftAddHeuristic),
        Action::Submit => app.enqueue_event(crate::app::AppEvent::DraftSubmit),
        Action::Backspace => app.enqueue_event(crate::app::AppEvent::DraftBackspace),
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
