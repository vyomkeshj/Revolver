use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct ImageItem {
    pub id: usize,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Heuristics {
    pub edge_threshold: f32,
    pub min_blob_area: usize,
    pub contrast_boost: f32,
}

#[derive(Clone, Debug)]
pub struct TaskDefinition {
    pub id: usize,
    pub name: String,
    pub dataset: Vec<ImageItem>,
    pub heuristics: Heuristics,
    pub max_iters: usize,
    pub created_at: SystemTime,
}

#[derive(Clone, Debug)]
pub struct Hypothesis {
    pub id: usize,
    pub description: String,
    pub score: f32,
}

#[derive(Clone, Debug)]
pub enum TaskStatus {
    Pending,
    Running,
    Cancelled,
    Done,
    Failed(String),
}

#[derive(Clone, Debug)]
pub enum TaskPhase {
    Defining,
    GeneratingHypotheses,
    EvaluatingHypotheses,
    Reducing,
    Synthesizing,
    Testing,
    Reporting,
    Finished,
}

#[derive(Clone, Debug)]
pub struct TaskSnapshot {
    pub id: usize,
    pub name: String,
    pub status: TaskStatus,
    pub phase: TaskPhase,
    pub progress: f32,
    pub iteration: usize,
    pub max_iters: usize,
    pub dataset_size: usize,
    pub heuristics: Heuristics,
    pub verified: Vec<Hypothesis>,
    pub discarded: Vec<Hypothesis>,
    pub best_score: f32,
    pub last_score: f32,
    pub report_path: Option<String>,
}

impl TaskDefinition {
    pub fn mock(id: usize, name: String) -> Self {
        let dataset = (0..16)
            .map(|i| ImageItem {
                id: i,
                name: format!("image_{id}_{i:02}.png"),
            })
            .collect::<Vec<_>>();

        let heuristics = Heuristics {
            edge_threshold: 0.42,
            min_blob_area: 120,
            contrast_boost: 1.25,
        };

        Self {
            id,
            name,
            dataset,
            heuristics,
            max_iters: 6,
            created_at: SystemTime::now(),
        }
    }
}

impl TaskSnapshot {
    pub fn from_definition(definition: &TaskDefinition) -> Self {
        Self {
            id: definition.id,
            name: definition.name.clone(),
            status: TaskStatus::Pending,
            phase: TaskPhase::Defining,
            progress: 0.0,
            iteration: 0,
            max_iters: definition.max_iters,
            dataset_size: definition.dataset.len(),
            heuristics: definition.heuristics.clone(),
            verified: Vec::new(),
            discarded: Vec::new(),
            best_score: 0.0,
            last_score: 0.0,
            report_path: None,
        }
    }
}
