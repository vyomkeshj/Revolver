use std::env;

use dotenvy::dotenv;
use rig::{client::{CompletionClient, ProviderClient}, completion::Prompt, providers::openai};

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
