use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::protocol::UiToEngine;
use crate::scheduler::TaskUpdate;
use crate::task::TaskSnapshot;
use crate::screens::{FragmentId, ScreenId};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppEvent {
    Quit,
    ToggleCursor,
    SwitchScreen(ScreenId),
    SwitchFragment(FragmentId),
    OpenTaskInput,
    CloseTaskInput,
    SelectTaskNext,
    SelectTaskPrev,
    CancelSelectedTask,
    DraftSwitchField,
    DraftFocusDescription,
    DraftFocusHypotheses,
    DraftCursorLeft,
    DraftCursorRight,
    DraftMoveUp,
    DraftMoveDown,
    DraftAddHeuristic,
    DraftAddImage,
    DraftBackspace,
    DraftInsertChar(char),
    DraftSubmit,
}

#[derive(Debug, Default)]
pub struct EventResult {
    pub quit: bool,
    pub cmd: Option<UiToEngine>,
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
    pub screen: ScreenId,
    pub fragment: FragmentId,
    pub draft: TaskDraft,
    pub cursor_pos: usize,
    pub cursor_visible: bool,
    event_queue: VecDeque<AppEvent>,
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
            screen: ScreenId::Main,
            fragment: FragmentId::MainTasks,
            draft,
            cursor_pos: 0,
            cursor_visible: true,
            event_queue: VecDeque::new(),
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

    pub fn set_fragment(&mut self, fragment: FragmentId) {
        self.fragment = fragment;
    }

    pub fn open_task_input(&mut self) {
        self.screen = ScreenId::TaskInput;
        self.fragment = FragmentId::TaskDescription;
        self.draft.field = DraftField::Name;
        self.draft.heuristics_focus = HeuristicsFocus::Titles;
        self.input = self.draft.name.clone();
        self.cursor_pos = self.input.len();
    }

    pub fn close_task_input(&mut self) {
        self.screen = ScreenId::Main;
        self.fragment = FragmentId::MainTasks;
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

    pub fn enqueue_event(&mut self, event: AppEvent) {
        self.event_queue.push_back(event);
    }

    pub fn pop_event(&mut self) -> Option<AppEvent> {
        self.event_queue.pop_front()
    }

    pub fn apply_event(&mut self, event: AppEvent) -> EventResult {
        let mut result = EventResult::default();
        match event {
            AppEvent::Quit => result.quit = true,
            AppEvent::ToggleCursor => self.toggle_cursor(),
            AppEvent::SwitchScreen(screen) => self.screen = screen,
            AppEvent::SwitchFragment(fragment) => self.fragment = fragment,
            AppEvent::OpenTaskInput => self.open_task_input(),
            AppEvent::CloseTaskInput => self.close_task_input(),
            AppEvent::SelectTaskNext => {
                if self.fragment == FragmentId::MainTasks {
                    self.select_next();
                }
            }
            AppEvent::SelectTaskPrev => {
                if self.fragment == FragmentId::MainTasks {
                    self.select_prev();
                }
            }
            AppEvent::CancelSelectedTask => {
                if let Some(task) = self.selected_task() {
                    result.cmd = Some(UiToEngine::CancelTask { id: task.id });
                }
            }
            AppEvent::DraftSwitchField => {
                if self.fragment == FragmentId::TaskDescription {
                    self.commit_draft_field();
                    self.draft.field = match self.draft.field {
                        DraftField::Name => DraftField::DatasetFolder,
                        DraftField::DatasetFolder => DraftField::Heuristics,
                        DraftField::Heuristics => DraftField::Name,
                    };
                    self.load_draft_field();
                }
            }
            AppEvent::DraftFocusDescription => self.set_fragment(FragmentId::TaskDescription),
            AppEvent::DraftFocusHypotheses => self.set_fragment(FragmentId::TaskHypotheses),
            AppEvent::DraftCursorLeft => self.move_cursor_left(),
            AppEvent::DraftCursorRight => self.move_cursor_right(),
            AppEvent::DraftMoveUp => self.move_selection_up(),
            AppEvent::DraftMoveDown => self.move_selection_down(),
            AppEvent::DraftAddHeuristic => {
                if self.fragment == FragmentId::TaskDescription
                    && self.draft.field == DraftField::Heuristics
                {
                    if self.draft.heuristics_focus == HeuristicsFocus::Images {
                        self.add_image();
                    } else {
                        let title = format!("Heuristic {}", self.draft.heuristics.len() + 1);
                        self.add_heuristic(title);
                    }
                }
            }
            AppEvent::DraftAddImage => self.add_image(),
            AppEvent::DraftBackspace => self.handle_backspace(),
            AppEvent::DraftInsertChar(ch) => self.handle_insert_char(ch),
            AppEvent::DraftSubmit => {
                if self.fragment == FragmentId::TaskDescription
                    && self.draft.field == DraftField::Heuristics
                    && self.draft.heuristics_focus == HeuristicsFocus::Images
                {
                    self.add_image();
                    return result;
                }
                self.commit_draft_field();
                let name = self.draft.name.trim().to_string();
                if !name.is_empty() {
                    result.cmd = Some(UiToEngine::AddTask { name });
                }
                self.reset_draft();
                self.close_task_input();
            }
        }
        result
    }

    fn move_cursor_left(&mut self) {
        if self.fragment == FragmentId::TaskDescription && self.draft.field == DraftField::Heuristics {
            if self.draft.heuristics_focus == HeuristicsFocus::Images {
                if self.cursor_pos == 0 {
                    self.draft.heuristics_focus = HeuristicsFocus::Titles;
                    self.cursor_pos = self
                        .draft
                        .heuristics
                        .get(self.draft.selected_heuristic)
                        .map(|h| h.title.len())
                        .unwrap_or(0);
                } else if let Some(image) = self
                    .draft
                    .heuristics
                    .get(self.draft.selected_heuristic)
                    .and_then(|h| h.images.get(self.draft.selected_image))
                {
                    self.cursor_pos = self.cursor_pos.saturating_sub(1).min(image.len());
                }
            } else if let Some(title) = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .map(|h| h.title.as_str())
            {
                self.cursor_pos = self.cursor_pos.saturating_sub(1).min(title.len());
            }
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field != DraftField::Heuristics
        {
            self.cursor_pos = self.cursor_pos.saturating_sub(1).min(self.input.len());
        }
    }

    fn move_cursor_right(&mut self) {
        if self.fragment == FragmentId::TaskDescription && self.draft.field == DraftField::Heuristics {
            if self.draft.heuristics_focus == HeuristicsFocus::Titles {
                if let Some(title) = self
                    .draft
                    .heuristics
                    .get(self.draft.selected_heuristic)
                    .map(|h| h.title.as_str())
                {
                    if self.cursor_pos >= title.len() {
                        self.draft.heuristics_focus = HeuristicsFocus::Images;
                        self.cursor_pos = self
                            .draft
                            .heuristics
                            .get(self.draft.selected_heuristic)
                            .and_then(|h| h.images.get(self.draft.selected_image))
                            .map(|s| s.len())
                            .unwrap_or(0);
                    } else {
                        self.cursor_pos = (self.cursor_pos + 1).min(title.len());
                    }
                }
            } else if let Some(image) = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .and_then(|h| h.images.get(self.draft.selected_image))
            {
                self.cursor_pos = (self.cursor_pos + 1).min(image.len());
            }
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field != DraftField::Heuristics
        {
            self.cursor_pos = (self.cursor_pos + 1).min(self.input.len());
        }
    }

    fn move_selection_up(&mut self) {
        if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Titles
            && self.draft.selected_heuristic > 0
        {
            self.draft.selected_heuristic -= 1;
            self.draft.selected_image = 0;
            self.cursor_pos = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .map(|h| h.title.len())
                .unwrap_or(0);
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Images
            && self.draft.selected_image > 0
        {
            self.draft.selected_image -= 1;
            self.cursor_pos = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .and_then(|h| h.images.get(self.draft.selected_image))
                .map(|s| s.len())
                .unwrap_or(0);
        } else if self.fragment == FragmentId::TaskHypotheses
            && self.draft.selected_hypothesis > 0
        {
            self.draft.selected_hypothesis -= 1;
        }
    }

    fn move_selection_down(&mut self) {
        if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Titles
        {
            let max = self.draft.heuristics.len().saturating_sub(1);
            self.draft.selected_heuristic = (self.draft.selected_heuristic + 1).min(max);
            self.draft.selected_image = 0;
            self.cursor_pos = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .map(|h| h.title.len())
                .unwrap_or(0);
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Images
        {
            let count = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .map(|h| h.images.len())
                .unwrap_or(0);
            let max = count.saturating_sub(1);
            self.draft.selected_image = (self.draft.selected_image + 1).min(max);
            self.cursor_pos = self
                .draft
                .heuristics
                .get(self.draft.selected_heuristic)
                .and_then(|h| h.images.get(self.draft.selected_image))
                .map(|s| s.len())
                .unwrap_or(0);
        } else if self.fragment == FragmentId::TaskHypotheses {
            let max = self.draft.hypotheses.len().saturating_sub(1);
            self.draft.selected_hypothesis =
                (self.draft.selected_hypothesis + 1).min(max);
        }
    }

    fn add_image(&mut self) {
        if let Some(heuristic) = self
            .draft
            .heuristics
            .get_mut(self.draft.selected_heuristic)
        {
            heuristic.images.push(String::new());
            self.draft.selected_image = heuristic.images.len().saturating_sub(1);
            self.cursor_pos = 0;
        }
    }

    fn handle_backspace(&mut self) {
        if self.fragment == FragmentId::TaskDescription && self.draft.field != DraftField::Heuristics {
            if self.cursor_pos > 0 {
                self.cursor_pos -= 1;
                self.input.remove(self.cursor_pos);
            }
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Titles
        {
            if let Some(heuristic) = self.draft.heuristics.get_mut(self.draft.selected_heuristic) {
                if self.cursor_pos > 0 && self.cursor_pos <= heuristic.title.len() {
                    self.cursor_pos -= 1;
                    heuristic.title.remove(self.cursor_pos);
                }
            }
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Images
        {
            if let Some(heuristic) = self.draft.heuristics.get_mut(self.draft.selected_heuristic) {
                if let Some(image) = heuristic.images.get_mut(self.draft.selected_image) {
                    if self.cursor_pos > 0 && self.cursor_pos <= image.len() {
                        self.cursor_pos -= 1;
                        image.remove(self.cursor_pos);
                    }
                }
            }
        }
    }

    fn handle_insert_char(&mut self, ch: char) {
        if self.fragment == FragmentId::TaskDescription && self.draft.field != DraftField::Heuristics {
            if self.cursor_pos <= self.input.len() {
                self.input.insert(self.cursor_pos, ch);
                self.cursor_pos += 1;
            }
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Titles
        {
            if let Some(heuristic) = self.draft.heuristics.get_mut(self.draft.selected_heuristic) {
                if self.cursor_pos <= heuristic.title.len() {
                    heuristic.title.insert(self.cursor_pos, ch);
                    self.cursor_pos += 1;
                }
            }
        } else if self.fragment == FragmentId::TaskDescription
            && self.draft.field == DraftField::Heuristics
            && self.draft.heuristics_focus == HeuristicsFocus::Images
        {
            if let Some(heuristic) = self.draft.heuristics.get_mut(self.draft.selected_heuristic) {
                if heuristic.images.is_empty() {
                    heuristic.images.push(String::new());
                    self.draft.selected_image = 0;
                    self.cursor_pos = 0;
                }
                if let Some(image) = heuristic.images.get_mut(self.draft.selected_image) {
                    if self.cursor_pos <= image.len() {
                        image.insert(self.cursor_pos, ch);
                        self.cursor_pos += 1;
                    }
                }
            }
        }
    }
}
