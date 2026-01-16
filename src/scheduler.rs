use std::collections::HashMap;
use std::time::Duration;

use rand::{rngs::StdRng, Rng, SeedableRng};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;

use crate::llm::RigLlm;
use crate::report::generate_markdown_report;
use crate::task::{
    Heuristics, Hypothesis, TaskDefinition, TaskPhase, TaskSnapshot, TaskStatus,
};

#[derive(Debug)]
pub enum SchedulerCommand {
    AddTask { name: String },
    CancelTask { id: usize },
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum TaskUpdate {
    Upsert(TaskSnapshot),
    Log { id: usize, message: String },
}

pub async fn run_scheduler(
    mut cmd_rx: mpsc::Receiver<SchedulerCommand>,
    ui_tx: mpsc::Sender<TaskUpdate>,
) {
    let mut next_id = 1usize;
    let mut cancels: HashMap<usize, watch::Sender<bool>> = HashMap::new();
    let llm = RigLlm::new_mock();

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            SchedulerCommand::AddTask { name } => {
                let id = next_id;
                next_id += 1;
                let definition = TaskDefinition::mock(id, name);
                let (cancel_tx, cancel_rx) = watch::channel(false);
                cancels.insert(id, cancel_tx);
                let llm_clone = llm.clone();
                let ui_tx_clone = ui_tx.clone();
                tokio::spawn(async move {
                    run_task(definition, cancel_rx, ui_tx_clone, llm_clone).await;
                });
            }
            SchedulerCommand::CancelTask { id } => {
                if let Some(cancel) = cancels.get(&id) {
                    let _ = cancel.send(true);
                }
            }
            SchedulerCommand::Shutdown => break,
        }
    }
}

async fn run_task(
    definition: TaskDefinition,
    cancel_rx: watch::Receiver<bool>,
    ui_tx: mpsc::Sender<TaskUpdate>,
    llm: RigLlm,
) {
    let mut snapshot = TaskSnapshot::from_definition(&definition);
    snapshot.status = TaskStatus::Running;
    snapshot.phase = TaskPhase::GeneratingHypotheses;
    send_update(&ui_tx, snapshot.clone()).await;
    send_log(&ui_tx, definition.id, "Task started with mock dataset.".to_string()).await;

    let mut verified: Vec<Hypothesis> = Vec::new();
    let mut discarded: Vec<Hypothesis> = Vec::new();
    let mut best_score = 0.0f32;
    let mut best_program = String::from("uninitialized");
    let mut no_improve_streak = 0usize;

    for iteration in 1..=definition.max_iters {
        if *cancel_rx.borrow() {
            snapshot.status = TaskStatus::Cancelled;
            snapshot.phase = TaskPhase::Finished;
            send_update(&ui_tx, snapshot.clone()).await;
            return;
        }

        snapshot.iteration = iteration;
        snapshot.phase = TaskPhase::GeneratingHypotheses;
        snapshot.progress = (iteration as f32 - 0.5) / definition.max_iters as f32;
        send_update(&ui_tx, snapshot.clone()).await;
        send_log(
            &ui_tx,
            definition.id,
            format!("Iteration {iteration}: generating hypotheses."),
        )
        .await;
        sleep(Duration::from_millis(250)).await;

        let hypothesis_texts = llm.generate_hypotheses(&definition, iteration).await;
        snapshot.phase = TaskPhase::EvaluatingHypotheses;
        send_update(&ui_tx, snapshot.clone()).await;
        send_log(
            &ui_tx,
            definition.id,
            format!("Iteration {iteration}: evaluating hypotheses."),
        )
        .await;

        let mut rng = StdRng::seed_from_u64(
            (definition.id as u64).wrapping_mul(31) ^ (iteration as u64).wrapping_mul(997),
        );

        let mut iteration_best = 0.0f32;
        for (idx, text) in hypothesis_texts.into_iter().enumerate() {
            let score = evaluate_hypothesis(
                &text,
                &definition.heuristics,
                definition.dataset.len(),
                &mut rng,
            );
            let hypothesis = Hypothesis {
                id: iteration * 100 + idx,
                description: text,
                score,
            };
            if score >= 0.6 {
                iteration_best = iteration_best.max(score);
                verified.push(hypothesis);
            } else {
                discarded.push(hypothesis);
            }
        }

        snapshot.phase = TaskPhase::Reducing;
        snapshot.verified = tail_hypotheses(&verified, 5);
        snapshot.discarded = tail_hypotheses(&discarded, 5);
        snapshot.last_score = iteration_best;
        send_update(&ui_tx, snapshot.clone()).await;
        sleep(Duration::from_millis(200)).await;

        snapshot.phase = TaskPhase::Synthesizing;
        if iteration_best > best_score + 0.02 {
            best_score = iteration_best;
            best_program = format!(
                "fn solve(images: &[Image]) -> Vec<Label> {{ /* v{iteration} */ }}"
            );
            no_improve_streak = 0;
            send_log(
                &ui_tx,
                definition.id,
                format!("Iteration {iteration}: new best score {:.2}.", best_score),
            )
            .await;
        } else {
            no_improve_streak += 1;
            send_log(
                &ui_tx,
                definition.id,
                format!("Iteration {iteration}: no improvement."),
            )
            .await;
        }
        snapshot.best_score = best_score;
        send_update(&ui_tx, snapshot.clone()).await;
        sleep(Duration::from_millis(200)).await;

        snapshot.phase = TaskPhase::Testing;
        snapshot.progress = iteration as f32 / definition.max_iters as f32;
        send_update(&ui_tx, snapshot.clone()).await;
        sleep(Duration::from_millis(200)).await;

        if no_improve_streak >= 2 {
            break;
        }
    }

    snapshot.phase = TaskPhase::Reporting;
    snapshot.status = TaskStatus::Running;
    send_update(&ui_tx, snapshot.clone()).await;
    send_log(&ui_tx, definition.id, "Generating report.".to_string()).await;

    match generate_markdown_report(&definition, &verified, &discarded, &best_program, best_score) {
        Ok(path) => {
            snapshot.report_path = Some(path);
            snapshot.status = TaskStatus::Done;
            snapshot.phase = TaskPhase::Finished;
        }
        Err(err) => {
            snapshot.status = TaskStatus::Failed(err);
            snapshot.phase = TaskPhase::Finished;
        }
    }

    snapshot.progress = 1.0;
    snapshot.verified = tail_hypotheses(&verified, 5);
    snapshot.discarded = tail_hypotheses(&discarded, 5);
    snapshot.best_score = best_score;
    send_update(&ui_tx, snapshot).await;
}

fn evaluate_hypothesis(
    hypothesis: &str,
    heuristics: &Heuristics,
    dataset_len: usize,
    rng: &mut StdRng,
) -> f32 {
    let hash = hypothesis
        .bytes()
        .fold(0u32, |acc, b| acc.wrapping_add(b as u32));
    let heuristic_bias =
        (heuristics.edge_threshold * 0.4) + (heuristics.contrast_boost * 0.2) +
        (heuristics.min_blob_area as f32 / 400.0);
    let dataset_bias = dataset_len as f32 / 25.0;
    let noise = rng.gen_range(-0.05..0.05);
    let score = (hash % 100) as f32 / 100.0;
    (0.4 * score + 0.4 * heuristic_bias + 0.2 * dataset_bias + noise).clamp(0.0, 1.0)
}

fn tail_hypotheses(all: &[Hypothesis], limit: usize) -> Vec<Hypothesis> {
    if all.len() <= limit {
        return all.to_vec();
    }
    all[all.len() - limit..].to_vec()
}

async fn send_update(ui_tx: &mpsc::Sender<TaskUpdate>, snapshot: TaskSnapshot) {
    let _ = ui_tx.send(TaskUpdate::Upsert(snapshot)).await;
}

async fn send_log(ui_tx: &mpsc::Sender<TaskUpdate>, id: usize, message: String) {
    let _ = ui_tx.send(TaskUpdate::Log { id, message }).await;
}
