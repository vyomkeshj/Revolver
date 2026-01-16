use std::collections::HashMap;

use crate::scheduler::TaskUpdate;
use crate::task::TaskSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activity {
    Main,
    TaskInput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fragment {
    MainTasks,
    MainDetail,
    MainInput,
    TaskDescription,
    TaskHypotheses,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftField {
    Name,
    DatasetFolder,
    Heuristics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeuristicsFocus {
    Titles,
    Images,
}

#[derive(Debug, Clone)]
pub struct HypothesisDraft {
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct HeuristicDraft {
    pub title: String,
    pub images: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct TaskDraft {
    pub name: String,
    pub dataset_folder: String,
    pub heuristics: Vec<HeuristicDraft>,
    pub hypotheses: Vec<HypothesisDraft>,
    pub field: DraftField,
    pub selected_hypothesis: usize,
    pub selected_heuristic: usize,
    pub selected_image: usize,
    pub heuristics_focus: HeuristicsFocus,
}

#[derive(Debug)]
pub struct AppState {
    tasks_by_id: HashMap<usize, TaskSnapshot>,
    order: Vec<usize>,
    pub selected: usize,
    pub input: String,
    pub activity: Activity,
    pub fragment: Fragment,
    pub draft: TaskDraft,
    pub cursor_pos: usize,
    pub cursor_visible: bool,
    logs: Vec<String>,
}

impl AppState {
    pub fn new() -> Self {
        let draft = TaskDraft {
            name: String::new(),
            dataset_folder: String::from("./datasets/mock"),
            heuristics: vec![
                HeuristicDraft {
                    title: "Edge threshold".to_string(),
                    images: vec!["/data/edge_01.png".to_string()],
                },
                HeuristicDraft {
                    title: "Blob area filter".to_string(),
                    images: vec![
                        "/data/blob_03.png".to_string(),
                        "/data/blob_07.png".to_string(),
                    ],
                },
            ],
            hypotheses: vec![
                HypothesisDraft {
                    title: "Edge + blob fusion".to_string(),
                },
                HypothesisDraft {
                    title: "Contrast boosted contours".to_string(),
                },
                HypothesisDraft {
                    title: "Texture density heuristic".to_string(),
                },
            ],
            field: DraftField::Name,
            selected_hypothesis: 0,
            selected_heuristic: 0,
            selected_image: 0,
            heuristics_focus: HeuristicsFocus::Titles,
        };
        Self {
            tasks_by_id: HashMap::new(),
            order: Vec::new(),
            selected: 0,
            input: String::new(),
            activity: Activity::Main,
            fragment: Fragment::MainTasks,
            draft,
            cursor_pos: 0,
            cursor_visible: true,
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

    pub fn set_fragment(&mut self, fragment: Fragment) {
        self.fragment = fragment;
    }

    pub fn open_task_input(&mut self) {
        self.activity = Activity::TaskInput;
        self.fragment = Fragment::TaskDescription;
        self.draft.field = DraftField::Name;
        self.draft.heuristics_focus = HeuristicsFocus::Titles;
        self.input = self.draft.name.clone();
        self.cursor_pos = self.input.len();
    }

    pub fn close_task_input(&mut self) {
        self.activity = Activity::Main;
        self.fragment = Fragment::MainTasks;
        self.input.clear();
        self.cursor_pos = 0;
    }

    pub fn commit_draft_field(&mut self) {
        match self.draft.field {
            DraftField::Name => self.draft.name = self.input.clone(),
            DraftField::DatasetFolder => self.draft.dataset_folder = self.input.clone(),
            DraftField::Heuristics => {}
        }
    }

    pub fn load_draft_field(&mut self) {
        self.input = match self.draft.field {
            DraftField::Name => self.draft.name.clone(),
            DraftField::DatasetFolder => self.draft.dataset_folder.clone(),
            DraftField::Heuristics => String::new(),
        };
        self.cursor_pos = self.input.len();
    }

    pub fn reset_draft(&mut self) {
        self.draft.name.clear();
        self.draft.dataset_folder = "./datasets/mock".to_string();
        self.draft.field = DraftField::Name;
        self.draft.selected_hypothesis = 0;
        self.draft.selected_heuristic = 0;
        self.draft.selected_image = 0;
        self.draft.heuristics_focus = HeuristicsFocus::Titles;
        self.input.clear();
        self.cursor_pos = 0;
    }

    pub fn add_heuristic(&mut self, title: String) {
        self.draft.heuristics.push(HeuristicDraft {
            title,
            images: vec!["/data/new_image.png".to_string()],
        });
        self.draft.selected_heuristic = self.draft.heuristics.len().saturating_sub(1);
        self.draft.selected_image = 0;
        self.cursor_pos = self
            .draft
            .heuristics
            .get(self.draft.selected_heuristic)
            .map(|h| h.title.len())
            .unwrap_or(0);
    }

    pub fn toggle_cursor(&mut self) {
        self.cursor_visible = !self.cursor_visible;
    }
}
