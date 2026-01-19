use std::env;

use dotenvy::dotenv;
use rig::{client::{CompletionClient, ProviderClient}, completion::Prompt, providers::openai};

use revolver::app::{AppEvent, AppState, MainScreenEvent, TaskInputEvent};

#[tokio::test]
async fn rig_llm_generation_uses_env_key() {
    let _ = dotenv();
    if env::var("OPENAI_API_KEY").is_err() {
        eprintln!("OPENAI_API_KEY not set in .env, skipping test.");
        return;
    }

    let client = openai::Client::from_env();
    let agent = client.agent("gpt-4o-mini").build();
    let response: String = agent
        .prompt("Reply with the single word: pong")
        .await
        .expect("LLM call failed");

    assert!(
        response.to_lowercase().contains("pong"),
        "Unexpected response: {response}"
    );
}

#[test]
fn replay_app_events_reaches_expected_state() {
    let mut app = AppState::new();
    let original_dataset = app.draft.dataset_folder.clone();
    let events = vec![
        AppEvent::Main(MainScreenEvent::OpenTaskInput),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('t')),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('a')),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('s')),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('k')),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('1')),
        AppEvent::TaskInput(TaskInputEvent::SwitchField),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('/')),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('d')),
        AppEvent::TaskInput(TaskInputEvent::InsertChar('s')),
    ];

    for event in events {
        let _ = app.apply_event(event);
    }

    assert_eq!(app.draft.name, "task1");
    assert!(app.draft.dataset_folder.starts_with(&original_dataset));
}
