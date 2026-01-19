mod app;
mod llm;
mod report;
mod scheduler;
mod task;
mod ui;
mod screens;

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};

use crate::app::AppState;
use crate::scheduler::{run_scheduler, SchedulerCommand};
use crate::screens::dispatch_key;

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
        terminal.draw(|frame| screens::draw(frame, &app))?;
        while event::poll(Duration::from_millis(0))? {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if dispatch_key(key.code, &mut app, &cmd_tx)? {
                    let _ = cmd_tx.send(SchedulerCommand::Shutdown).await;
                    restore_terminal(&mut terminal)?;
                    return Ok(());
                }
            }
        }

        tokio::select! {
            _ = tick.tick() => {
                app.toggle_cursor();
            }
            Some(update) = ui_rx.recv() => {
                app.apply_update(update);
            }
        }
    }
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