pub mod common;
pub mod main;
pub mod task_input;

use crossterm::event::KeyCode;
use ratatui::Frame;
use crate::app::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScreenId {
    Main,
    TaskInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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
) -> std::io::Result<bool> {
    current_screen(app).handle_key(key, app)
}

pub fn draw(frame: &mut Frame, app: &AppState) {
    current_screen(app).draw(frame, app);
}
