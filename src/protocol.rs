use serde::{Deserialize, Serialize};

use crate::scheduler::TaskUpdate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiToEngine {
    AddTask { name: String },
    CancelTask { id: usize },
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineToUi {
    TaskUpdate(TaskUpdate),
}
