use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::UNIX_EPOCH;

use crate::task::{Hypothesis, TaskDefinition};

pub fn generate_markdown_report(
    definition: &TaskDefinition,
    verified: &[Hypothesis],
    discarded: &[Hypothesis],
    program: &str,
    best_score: f32,
) -> Result<String, String> {
    let mut report = String::new();
    report.push_str(&format!("# Task Report: {}\n\n", definition.name));
    report.push_str("## Task Definition\n");
    report.push_str(&format!("- Task id: {}\n", definition.id));
    if let Ok(duration) = definition.created_at.duration_since(UNIX_EPOCH) {
        report.push_str(&format!("- Created (unix): {}\n", duration.as_secs()));
    }
    report.push_str(&format!("- Dataset size: {}\n", definition.dataset.len()));
    report.push_str(&format!(
        "- Heuristics: edge_threshold={:.2}, min_blob_area={}, contrast_boost={:.2}\n",
        definition.heuristics.edge_threshold,
        definition.heuristics.min_blob_area,
        definition.heuristics.contrast_boost
    ));
    report.push_str(&format!("- Max iterations: {}\n\n", definition.max_iters));

    report.push_str("### Dataset sample\n");
    if definition.dataset.is_empty() {
        report.push_str("- _empty_\n\n");
    } else {
        for image in definition.dataset.iter().take(8) {
            report.push_str(&format!("- {}: {}\n", image.id, image.name));
        }
        report.push('\n');
    }

    report.push_str("## Summary\n");
    report.push_str(&format!("- Best score: {:.3}\n", best_score));
    report.push_str(&format!("- Verified hypotheses: {}\n", verified.len()));
    report.push_str(&format!("- Discarded hypotheses: {}\n\n", discarded.len()));

    report.push_str("## Verified Hypotheses\n");
    if verified.is_empty() {
        report.push_str("- _None_\n");
    } else {
        for hypothesis in verified.iter().take(20) {
            report.push_str(&format!(
                "- [{}] {} (score {:.2})\n",
                hypothesis.id, hypothesis.description, hypothesis.score
            ));
        }
    }
    report.push('\n');

    report.push_str("## Discarded Hypotheses\n");
    if discarded.is_empty() {
        report.push_str("- _None_\n");
    } else {
        for hypothesis in discarded.iter().take(20) {
            report.push_str(&format!(
                "- [{}] {} (score {:.2})\n",
                hypothesis.id, hypothesis.description, hypothesis.score
            ));
        }
    }
    report.push('\n');

    report.push_str("## Synthesized Program (mock)\n");
    report.push_str("```rust\n");
    report.push_str(program);
    report.push_str("\n```\n");

    let mut path = PathBuf::from("reports");
    create_dir_all(&path).map_err(|e| e.to_string())?;
    let sanitized = sanitize_filename(&definition.name);
    path.push(format!("task_{}_{}.md", definition.id, sanitized));
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(report.as_bytes()).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

fn sanitize_filename(input: &str) -> String {
    let mut out = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if ch == ' ' || ch == '-' || ch == '_' {
            out.push('_');
        }
    }
    if out.is_empty() {
        "task".to_string()
    } else {
        out
    }
}
