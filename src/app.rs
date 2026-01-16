use std::collections::HashMap;

use crate::scheduler::TaskUpdate;
use crate::task::TaskSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    Tasks,
    Detail,
    Input,
}

#[derive(Debug)]
pub struct AppState {
    tasks_by_id: HashMap<usize, TaskSnapshot>,
    order: Vec<usize>,
    pub selected: usize,
    pub input_mode: bool,
    pub input: String,
    pub focus: FocusSection,
    logs: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            tasks_by_id: HashMap::new(),
            order: Vec::new(),
            selected: 0,
            input_mode: false,
            input: String::new(),
            focus: FocusSection::Tasks,
            logs: Vec::new(),
        }
    }

    pub fn apply_update(&mut self, update: TaskUpdate) {
        match update {
            TaskUpdate::Upsert(snapshot) => {
                let id = snapshot.id;
                let is_new = !self.tasks_by_id.contains_key(&id);
                self.tasks_by_id.insert(id, snapshot);
                if is_new {
                    self.order.push(id);
                }
                if self.selected >= self.order.len() && !self.order.is_empty() {
                    self.selected = self.order.len() - 1;
                }
            }
            TaskUpdate::Log { id, message } => {
                self.logs.push(format!("Task {id}: {message}"));
                if self.logs.len() > 5 {
                    self.logs.remove(0);
                }
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.order.is_empty() {
            return;
        }
        self.selected = (self.selected + 1).min(self.order.len() - 1);
    }

    pub fn select_prev(&mut self) {
        if self.order.is_empty() {
            return;
        }
        if self.selected == 0 {
            return;
        }
        self.selected -= 1;
    }

    pub fn tasks_in_order(&self) -> Vec<TaskSnapshot> {
        self.order
            .iter()
            .filter_map(|id| self.tasks_by_id.get(id).cloned())
            .collect()
    }

    pub fn selected_task(&self) -> Option<TaskSnapshot> {
        self.order
            .get(self.selected)
            .and_then(|id| self.tasks_by_id.get(id))
            .cloned()
    }

    pub fn recent_logs(&self) -> &[String] {
        &self.logs
    }

    pub fn set_focus(&mut self, focus: FocusSection) {
        self.focus = focus;
    }
}
