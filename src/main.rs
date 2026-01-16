mod app;
mod llm;
mod report;
mod scheduler;
mod task;
mod ui;

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};

use crate::app::{AppState, FocusSection};
use crate::scheduler::{run_scheduler, SchedulerCommand};

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;

    terminal.draw(|frame| ui::draw_splash(frame))?;
    sleep(Duration::from_secs(3)).await;

    let (cmd_tx, cmd_rx) = mpsc::channel(32);
    let (ui_tx, mut ui_rx) = mpsc::channel(128);
    tokio::spawn(run_scheduler(cmd_rx, ui_tx));

    let mut app = AppState::new();
    let mut tick = interval(Duration::from_millis(200));

    loop {
        terminal.draw(|frame| ui::draw(frame, &app))?;
        while event::poll(Duration::from_millis(0))? {
            let event = event::read()?;
            if handle_event(event, &mut app, &cmd_tx).await? {
                let _ = cmd_tx.send(SchedulerCommand::Shutdown).await;
                restore_terminal(&mut terminal)?;
                return Ok(());
            }
        }

        tokio::select! {
            _ = tick.tick() => {}
            Some(update) = ui_rx.recv() => {
                app.apply_update(update);
            }
        }
    }
}

async fn handle_event(
    event: Event,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> io::Result<bool> {
    if let Event::Key(key) = event {
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }
        if app.input_mode {
            match key.code {
                KeyCode::Esc => {
                    app.input_mode = false;
                    app.input.clear();
                    app.set_focus(FocusSection::Tasks);
                }
                KeyCode::Enter => {
                    let name = app.input.trim().to_string();
                    if !name.is_empty() {
                        let _ = cmd_tx.send(SchedulerCommand::AddTask { name }).await;
                    }
                    app.input_mode = false;
                    app.input.clear();
                    app.set_focus(FocusSection::Tasks);
                }
                KeyCode::Backspace => {
                    app.input.pop();
                }
                KeyCode::Char(ch) => {
                    app.input.push(ch);
                }
                _ => {}
            }
            return Ok(false);
        }

        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n') => {
                app.input_mode = true;
                app.set_focus(FocusSection::Input);
            }
            KeyCode::Char('t') | KeyCode::Char('T') => {
                app.set_focus(FocusSection::Tasks);
            }
            KeyCode::Char('d') | KeyCode::Char('D') => {
                app.set_focus(FocusSection::Detail);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if app.focus == FocusSection::Tasks {
                    app.select_next();
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if app.focus == FocusSection::Tasks {
                    app.select_prev();
                }
            }
            KeyCode::Char('c') => {
                if let Some(task) = app.selected_task() {
                    let _ = cmd_tx
                        .send(SchedulerCommand::CancelTask { id: task.id })
                        .await;
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}