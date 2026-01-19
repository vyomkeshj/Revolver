pub mod fragments;

use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::app::AppState;
use crate::scheduler::SchedulerCommand;
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
    Ok(false)
}

fn handle_action(
    action: Action,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> std::io::Result<bool> {
    match action {
        Action::Quit => return Ok(true),
        Action::OpenTaskInput => app.open_task_input(),
        Action::FocusTasks => app.set_fragment(FragmentId::MainTasks),
        Action::FocusDetail => app.set_fragment(FragmentId::MainDetail),
        Action::FocusInput => app.set_fragment(FragmentId::MainInput),
        Action::NextTask => {
            if app.fragment == FragmentId::MainTasks {
                app.select_next();
            }
        }
        Action::PrevTask => {
            if app.fragment == FragmentId::MainTasks {
                app.select_prev();
            }
        }
        Action::CancelTask => {
            if let Some(task) = app.selected_task() {
                let _ = cmd_tx.try_send(SchedulerCommand::CancelTask { id: task.id });
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
