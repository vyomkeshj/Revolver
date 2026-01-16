use rand::{rngs::StdRng, Rng, SeedableRng};

use crate::task::TaskDefinition;

#[derive(Clone, Debug)]
pub struct RigLlm {
    seed: u64,
}

impl RigLlm {
    pub fn new_mock() -> Self {
        Self { seed: 42 }
    }

    pub async fn generate_hypotheses(
        &self,
        task: &TaskDefinition,
        iteration: usize,
    ) -> Vec<String> {
        #[cfg(feature = "real-llm")]
        {
            use rig::{client::CompletionClient, completion::Prompt, providers::openai};

            let client = openai::Client::from_env();
            let agent = client.agent("gpt-4o-mini").build();
            let prompt = format!(
                "Generate 5 short hypotheses for task '{}' using heuristics {:?} on {} images.",
                task.name,
                task.heuristics,
                task.dataset.len()
            );

            if let Ok(response) = agent.prompt(prompt).await {
                let lines = response
                    .lines()
                    .map(|line| line.trim().trim_start_matches('-').trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string())
                    .collect::<Vec<_>>();
                if !lines.is_empty() {
                    return lines;
                }
            }
        }

        let mut rng = StdRng::seed_from_u64(
            self.seed ^ (task.id as u64) ^ (iteration as u64).saturating_mul(9_973),
        );
        let count = rng.gen_range(4..=7);
        (0..count)
            .map(|i| {
                format!(
                    "Iter {iteration} hypothesis {i}: focus on blobs>={} and edge {:.2}",
                    task.heuristics.min_blob_area,
                    task.heuristics.edge_threshold
                )
            })
            .collect()
    }
}
