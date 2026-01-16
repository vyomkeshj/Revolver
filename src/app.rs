use std::collections::HashMap;

use crate::scheduler::TaskUpdate;
use crate::task::TaskSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    Tasks,
    Detail,
    Input,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    TaskInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftFocus {
    Fields,
    Hypotheses,
    Images,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftField {
    Name,
    DatasetFolder,
}

#[derive(Debug, Clone)]
pub struct HypothesisDraft {
    pub title: String,
    pub images: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TaskDraft {
    pub name: String,
    pub dataset_folder: String,
    pub hypotheses: Vec<HypothesisDraft>,
    pub field: DraftField,
    pub focus: DraftFocus,
    pub selected_hypothesis: usize,
    pub selected_image: usize,
}

#[derive(Debug)]
pub struct AppState {
    tasks_by_id: HashMap<usize, TaskSnapshot>,
    order: Vec<usize>,
    pub selected: usize,
    pub input_mode: bool,
    pub input: String,
    pub focus: FocusSection,
    pub screen: Screen,
    pub draft: TaskDraft,
    logs: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        let draft = TaskDraft {
            name: String::new(),
            dataset_folder: String::from("./datasets/mock"),
            hypotheses: vec![
                HypothesisDraft {
                    title: "Edge + blob fusion".to_string(),
                    images: vec![
                        "image_00.png".to_string(),
                        "image_03.png".to_string(),
                        "image_07.png".to_string(),
                    ],
                },
                HypothesisDraft {
                    title: "Contrast boosted contours".to_string(),
                    images: vec![
                        "image_02.png".to_string(),
                        "image_05.png".to_string(),
                        "image_11.png".to_string(),
                    ],
                },
                HypothesisDraft {
                    title: "Texture density heuristic".to_string(),
                    images: vec![
                        "image_04.png".to_string(),
                        "image_08.png".to_string(),
                    ],
                },
            ],
            field: DraftField::Name,
            focus: DraftFocus::Fields,
            selected_hypothesis: 0,
            selected_image: 0,
        };
        Self {
            tasks_by_id: HashMap::new(),
            order: Vec::new(),
            selected: 0,
            input_mode: false,
            input: String::new(),
            focus: FocusSection::Tasks,
            screen: Screen::Main,
            draft,
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

    pub fn open_task_input(&mut self) {
        self.screen = Screen::TaskInput;
        self.input_mode = true;
        self.focus = FocusSection::Input;
        self.draft.field = DraftField::Name;
        self.draft.focus = DraftFocus::Fields;
        self.input = self.draft.name.clone();
    }

    pub fn close_task_input(&mut self) {
        self.screen = Screen::Main;
        self.input_mode = false;
        self.focus = FocusSection::Tasks;
        self.input.clear();
    }

    pub fn commit_draft_field(&mut self) {
        match self.draft.field {
            DraftField::Name => self.draft.name = self.input.clone(),
            DraftField::DatasetFolder => self.draft.dataset_folder = self.input.clone(),
        }
    }

    pub fn load_draft_field(&mut self) {
        self.input = match self.draft.field {
            DraftField::Name => self.draft.name.clone(),
            DraftField::DatasetFolder => self.draft.dataset_folder.clone(),
        };
    }

    pub fn reset_draft(&mut self) {
        self.draft.name.clear();
        self.draft.dataset_folder = "./datasets/mock".to_string();
        self.draft.field = DraftField::Name;
        self.draft.focus = DraftFocus::Fields;
        self.draft.selected_hypothesis = 0;
        self.draft.selected_image = 0;
        self.input.clear();
    }
}
