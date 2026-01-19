pub mod fragments;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::{AppState, TaskInputEvent, TextEditEvent};
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
    ) -> std::io::Result<bool> {
        handle_key(key, app)
    }
}

pub fn handle_key(
    key: KeyCode,
    app: &mut AppState,
) -> std::io::Result<bool> {
    if let Some(binding) = KEY_BINDINGS.iter().find(|b| b.key == key) {
        return handle_action(binding.action, app);
    }
    if let KeyCode::Char(ch) = key {
        app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Edit(
            TextEditEvent::InsertChar(ch),
        )));
        return Ok(false);
    }
    Ok(false)
}

fn handle_action(
    action: Action,
    app: &mut AppState,
) -> std::io::Result<bool> {
    match action {
        Action::Close => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Close)),
        Action::SwitchField => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::SwitchField)),
        Action::FocusDescription => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::FocusDescription)),
        Action::FocusHypotheses => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::FocusHypotheses)),
        Action::CursorLeft => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Edit(TextEditEvent::CursorLeft))),
        Action::CursorRight => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Edit(TextEditEvent::CursorRight))),
        Action::MoveUp => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Edit(TextEditEvent::MoveUp))),
        Action::MoveDown => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Edit(TextEditEvent::MoveDown))),
        Action::AddHeuristic => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::AddHeuristic)),
        Action::Submit => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Submit)),
        Action::Backspace => app.enqueue_event(crate::app::AppEvent::TaskInput(TaskInputEvent::Edit(TextEditEvent::Backspace))),
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
