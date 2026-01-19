pub mod fragments;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::{AppEvent, AppState};
use crate::screens::{FragmentId, KeyBinding, Screen};

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Quit,
    OpenTaskInput,
    FocusTasks,
    FocusDetail,
    FocusInput,
    NextTask,
    PrevTask,
    CancelTask,
}

const KEY_BINDINGS: &[KeyBinding<Action>] = &[
    KeyBinding {
        key: KeyCode::Char('q'),
        action: Action::Quit,
    },
    KeyBinding {
        key: KeyCode::Char('n'),
        action: Action::OpenTaskInput,
    },
    KeyBinding {
        key: KeyCode::Char('t'),
        action: Action::FocusTasks,
    },
    KeyBinding {
        key: KeyCode::Char('d'),
        action: Action::FocusDetail,
    },
    KeyBinding {
        key: KeyCode::Char('i'),
        action: Action::FocusInput,
    },
    KeyBinding {
        key: KeyCode::Down,
        action: Action::NextTask,
    },
    KeyBinding {
        key: KeyCode::Up,
        action: Action::PrevTask,
    },
    KeyBinding {
        key: KeyCode::Char('j'),
        action: Action::NextTask,
    },
    KeyBinding {
        key: KeyCode::Char('k'),
        action: Action::PrevTask,
    },
    KeyBinding {
        key: KeyCode::Char('c'),
        action: Action::CancelTask,
    },
];

pub struct MainScreen;

impl Screen for MainScreen {
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
    Ok(false)
}

fn handle_action(
    action: Action,
    app: &mut AppState,
) -> std::io::Result<bool> {
    match action {
        Action::Quit => app.enqueue_event(AppEvent::Quit),
        Action::OpenTaskInput => app.enqueue_event(AppEvent::OpenTaskInput),
        Action::FocusTasks => app.enqueue_event(AppEvent::SwitchFragment(FragmentId::MainTasks)),
        Action::FocusDetail => app.enqueue_event(AppEvent::SwitchFragment(FragmentId::MainDetail)),
        Action::FocusInput => app.enqueue_event(AppEvent::SwitchFragment(FragmentId::MainInput)),
        Action::NextTask => app.enqueue_event(AppEvent::SelectTaskNext),
        Action::PrevTask => app.enqueue_event(AppEvent::SelectTaskPrev),
        Action::CancelTask => app.enqueue_event(AppEvent::CancelSelectedTask),
    }
    Ok(false)
}

pub fn draw(frame: &mut Frame, app: &AppState) {
    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(frame.size());

    fragments::header::draw(frame, root[0]);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(root[1]);
    fragments::tasks::draw(frame, main[0], app);
    fragments::detail::draw(frame, main[1], app);

    fragments::input::draw(frame, root[2], app);
    fragments::help::draw(frame, root[3]);
}
