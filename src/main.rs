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

use crate::app::{Activity, AppState, DraftField, Fragment, HeuristicsFocus};
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
            _ = tick.tick() => {
                app.toggle_cursor();
            }
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
        if app.activity == Activity::TaskInput {
            return handle_task_input(key.code, app, cmd_tx).await;
        }
        return handle_main_activity(key.code, app, cmd_tx).await;
    }
    Ok(false)
}

async fn handle_main_activity(
    code: KeyCode,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> io::Result<bool> {
    match code {
        KeyCode::Char('q') => return Ok(true),
        KeyCode::Char('n') => {
            app.open_task_input();
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            app.set_fragment(Fragment::MainTasks);
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            app.set_fragment(Fragment::MainDetail);
        }
        KeyCode::Char('i') | KeyCode::Char('I') => {
            app.set_fragment(Fragment::MainInput);
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.fragment == Fragment::MainTasks {
                app.select_next();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.fragment == Fragment::MainTasks {
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
    Ok(false)
}

async fn handle_task_input(
    code: KeyCode,
    app: &mut AppState,
    cmd_tx: &mpsc::Sender<SchedulerCommand>,
) -> io::Result<bool> {
    match code {
        KeyCode::Esc => {
            app.close_task_input();
        }
        KeyCode::Tab => {
            if app.fragment == Fragment::TaskDescription {
                app.commit_draft_field();
                app.draft.field = match app.draft.field {
                    DraftField::Name => DraftField::DatasetFolder,
                    DraftField::DatasetFolder => DraftField::Heuristics,
                    DraftField::Heuristics => DraftField::Name,
                };
                app.load_draft_field();
            }
        }
        KeyCode::F(1) => app.set_fragment(Fragment::TaskDescription),
        KeyCode::F(2) => app.set_fragment(Fragment::TaskHypotheses),
        KeyCode::Right => {
            if app.fragment == Fragment::TaskDescription
                && app.draft.field == DraftField::Heuristics
            {
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
            } else if app.fragment == Fragment::TaskDescription
                && app.draft.field != DraftField::Heuristics
            {
                app.cursor_pos = (app.cursor_pos + 1).min(app.input.len());
            }
        }
        KeyCode::Left => {
            if app.fragment == Fragment::TaskDescription
                && app.draft.field == DraftField::Heuristics
            {
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
            } else if app.fragment == Fragment::TaskDescription
                && app.draft.field != DraftField::Heuristics
            {
                app.cursor_pos = app.cursor_pos.saturating_sub(1).min(app.input.len());
            }
        }
        KeyCode::Up => {
            if app.fragment == Fragment::TaskDescription
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
            } else if app.fragment == Fragment::TaskDescription
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
            } else if app.fragment == Fragment::TaskHypotheses
                && app.draft.selected_hypothesis > 0
            {
                app.draft.selected_hypothesis -= 1;
            }
        }
        KeyCode::Down => {
            if app.fragment == Fragment::TaskDescription
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
            } else if app.fragment == Fragment::TaskDescription
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
            } else if app.fragment == Fragment::TaskHypotheses {
                let max = app.draft.hypotheses.len().saturating_sub(1);
                app.draft.selected_hypothesis =
                    (app.draft.selected_hypothesis + 1).min(max);
            }
        }
        KeyCode::Char('+') => {
            if app.fragment == Fragment::TaskDescription
                && app.draft.field == DraftField::Heuristics
            {
                let title = format!("Heuristic {}", app.draft.heuristics.len() + 1);
                app.add_heuristic(title);
            }
        }
        KeyCode::Enter => {
            if app.fragment == Fragment::TaskDescription
                && app.draft.field == DraftField::Heuristics
                && app.draft.heuristics_focus == HeuristicsFocus::Images
            {
                if let Some(heuristic) = app
                    .draft
                    .heuristics
                    .get_mut(app.draft.selected_heuristic)
                {
                    heuristic.images.push(String::new());
                    app.draft.selected_image = heuristic.images.len().saturating_sub(1);
                    app.cursor_pos = 0;
                }
                return Ok(false);
            }
            app.commit_draft_field();
            let name = app.draft.name.trim().to_string();
            if !name.is_empty() {
                let _ = cmd_tx.send(SchedulerCommand::AddTask { name }).await;
            }
            app.reset_draft();
            app.close_task_input();
        }
        KeyCode::Backspace => {
            if app.fragment == Fragment::TaskDescription && app.draft.field != DraftField::Heuristics {
                if app.cursor_pos > 0 {
                    app.cursor_pos -= 1;
                    app.input.remove(app.cursor_pos);
                }
            } else if app.fragment == Fragment::TaskDescription
                && app.draft.field == DraftField::Heuristics
                && app.draft.heuristics_focus == HeuristicsFocus::Titles
            {
                if let Some(heuristic) = app.draft.heuristics.get_mut(app.draft.selected_heuristic) {
                    if app.cursor_pos > 0 && app.cursor_pos <= heuristic.title.len() {
                        app.cursor_pos -= 1;
                        heuristic.title.remove(app.cursor_pos);
                    }
                }
            } else if app.fragment == Fragment::TaskDescription
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
        KeyCode::Char(ch) => {
            if app.fragment == Fragment::TaskDescription && app.draft.field != DraftField::Heuristics {
                if app.cursor_pos <= app.input.len() {
                    app.input.insert(app.cursor_pos, ch);
                    app.cursor_pos += 1;
                }
            } else if app.fragment == Fragment::TaskDescription
                && app.draft.field == DraftField::Heuristics
                && app.draft.heuristics_focus == HeuristicsFocus::Titles
            {
                if let Some(heuristic) = app.draft.heuristics.get_mut(app.draft.selected_heuristic) {
                    if app.cursor_pos <= heuristic.title.len() {
                        heuristic.title.insert(app.cursor_pos, ch);
                        app.cursor_pos += 1;
                    }
                }
            } else if app.fragment == Fragment::TaskDescription
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
        }
        _ => {}
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