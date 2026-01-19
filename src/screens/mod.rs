pub mod common;
pub mod main;
pub mod task_input;

use crossterm::event::KeyCode;
use ratatui::Frame;
use tokio::sync::mpsc;

use crate::app::AppState;
use crate::scheduler::SchedulerCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenId {
    Main,
    TaskInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentId {
    MainTasks,
    MainDetail,
    MainInput,
    TaskDescription,
    TaskHypotheses,
}

#[derive(Debug, Clone, Copy)]
pub struct KeyBinding<A: Copy> {
    pub key: KeyCode,
    pub action: A,
}

pub trait Screen {
    fn draw(&self, frame: &mut Frame, app: &AppState);
    fn handle_key(
        &self,
        key: KeyCode,
        app: &mut AppState,
        cmd_tx: &mpsc::Sender<SchedulerCommand>,
    ) -> std::io::Result<bool>;
}

static MAIN_SCREEN: main::MainScreen = main::MainScreen;
static TASK_INPUT_SCREEN: task_input::TaskInputScreen = task_input::TaskInputScreen;

fn current_screen(app: &AppState) -> &'static dyn Screen {
    match app.screen {
        ScreenId::Main => &MAIN_SCREEN,
        ScreenId::TaskInput => &TASK_INPUT_SCREEN,
    }
}

pub fn dispatch_key(
    key: KeyCode,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> std::io::Result<bool> {
    current_screen(app).handle_key(key, app, cmd_tx)
}

pub fn draw(frame: &mut Frame, app: &AppState) {
    current_screen(app).draw(frame, app);
}
