use crate::format::ConfigItem;
use tui_input::Input;

/// Represents the current operating mode of the application.
pub enum Mode {
    /// Standard navigation and command mode.
    Normal,
    /// Active text entry mode for modifying values.
    Insert,
    /// Active search mode for filtering keys.
    Search,
}

/// The core application state, holding all configuration variables and UI status.
pub struct App {
    /// The list of configuration variables being edited.
    pub vars: Vec<ConfigItem>,
    /// Index of the currently selected variable in the list.
    pub selected: usize,
    /// The current interaction mode (Normal or Insert).
    pub mode: Mode,
    /// Whether the main application loop should continue running.
    pub running: bool,
    /// An optional message to display in the status bar (e.g., "Saved to .env").
    pub status_message: Option<String>,
    /// The active text input buffer for the selected variable.
    pub input: Input,
    /// The current search query for filtering keys.
    pub search_query: String,
}

impl App {
    /// Initializes a new application instance with the provided variables.
    pub fn new(vars: Vec<ConfigItem>) -> Self {
        let initial_input = vars.get(0).and_then(|v| v.value.clone()).unwrap_or_default();
        Self {
            vars,
            selected: 0,
            mode: Mode::Normal,
            running: true,
            status_message: None,
            input: Input::new(initial_input),
            search_query: String::new(),
        }
    }

    /// Returns the indices of variables that match the search query.
    pub fn matching_indices(&self) -> Vec<usize> {
        if self.search_query.is_empty() {
            return Vec::new();
        }
        let query = self.search_query.to_lowercase();
        self.vars
            .iter()
            .enumerate()
            .filter(|(_, v)| v.key.to_lowercase().contains(&query))
            .map(|(i, _)| i)
            .collect()
    }

    /// Moves the selection to the next variable in the list, wrapping around if necessary.
    pub fn next(&mut self) {
        if !self.vars.is_empty() {
            self.selected = (self.selected + 1) % self.vars.len();
            self.sync_input_with_selected();
        }
    }

    /// Moves the selection to the previous variable in the list, wrapping around if necessary.
    pub fn previous(&mut self) {
        if !self.vars.is_empty() {
            if self.selected == 0 {
                self.selected = self.vars.len() - 1;
            } else {
                self.selected -= 1;
            }
            self.sync_input_with_selected();
        }
    }

    /// Jumps to the top of the list.
    pub fn jump_top(&mut self) {
        if !self.vars.is_empty() {
            self.selected = 0;
            self.sync_input_with_selected();
        }
    }

    /// Jumps to the bottom of the list.
    pub fn jump_bottom(&mut self) {
        if !self.vars.is_empty() {
            self.selected = self.vars.len() - 1;
            self.sync_input_with_selected();
        }
    }

    /// Jumps to the next variable that matches the search query.
    pub fn jump_next_match(&mut self) {
        let indices = self.matching_indices();
        if indices.is_empty() {
            return;
        }

        let next_match = indices
            .iter()
            .find(|&&i| i > self.selected)
            .or_else(|| indices.first());

        if let Some(&index) = next_match {
            self.selected = index;
            self.sync_input_with_selected();
        }
    }

    /// Jumps to the previous variable that matches the search query.
    pub fn jump_previous_match(&mut self) {
        let indices = self.matching_indices();
        if indices.is_empty() {
            return;
        }

        let prev_match = indices
            .iter()
            .rev()
            .find(|&&i| i < self.selected)
            .or_else(|| indices.last());

        if let Some(&index) = prev_match {
            self.selected = index;
            self.sync_input_with_selected();
        }
    }

    /// Updates the input buffer to reflect the value of the currently selected variable.
    pub fn sync_input_with_selected(&mut self) {
        if let Some(var) = self.vars.get(self.selected) {
            let val = var.value.clone().unwrap_or_default();
            self.input = Input::new(val);
        }
    }

    /// Commits the current text in the input buffer back to the selected variable's value.
    pub fn commit_input(&mut self) {
        if let Some(var) = self.vars.get_mut(self.selected) {
            if !var.is_group {
                var.value = Some(self.input.value().to_string());
                var.status = crate::format::ItemStatus::Modified;
            }
        }
    }

    /// Transitions the application into Insert Mode.
    pub fn enter_insert(&mut self) {
        if let Some(var) = self.vars.get(self.selected) {
            if !var.is_group {
                self.mode = Mode::Insert;
                self.status_message = None;
            }
        }
    }

    /// Commits the current input and transitions the application into Normal Mode.
    pub fn enter_normal(&mut self) {
        self.commit_input();
        self.mode = Mode::Normal;
    }

    /// Adds a new item to an array if the selected item is part of one.
    pub fn add_array_item(&mut self, after: bool) {
        if self.vars.is_empty() {
            return;
        }

        let (base, idx, depth) = {
            let selected_item = &self.vars[self.selected];
            if selected_item.is_group {
                return;
            }
            let path = &selected_item.path;
            if let Some((base, idx)) = parse_index(path) {
                (base.to_string(), idx, selected_item.depth)
            } else {
                return;
            }
        };

        let new_idx = if after { idx + 1 } else { idx };
        let insert_pos = if after {
            self.selected + 1
        } else {
            self.selected
        };

        // 1. Shift all items in this array that have index >= new_idx
        for var in self.vars.iter_mut() {
            if var.path.starts_with(&base) {
                if let Some((b, i)) = parse_index(&var.path) {
                    if b == base && i >= new_idx {
                        var.path = format!("{}[{}]", base, i + 1);
                        // Also update key if it was just the index
                        if var.key == format!("[{}]", i) {
                            var.key = format!("[{}]", i + 1);
                        }
                    }
                }
            }
        }

        // 2. Insert new item
        let new_item = ConfigItem {
            key: format!("[{}]", new_idx),
            path: format!("{}[{}]", base, new_idx),
            value: Some("".to_string()),
            template_value: None,
            default_value: None,
            depth,
            is_group: false,
            status: crate::format::ItemStatus::Modified,
            value_type: crate::format::ValueType::String,
        };
        self.vars.insert(insert_pos, new_item);
        self.selected = insert_pos;
        self.sync_input_with_selected();
        self.mode = Mode::Insert;
        self.status_message = None;
    }
}

fn parse_index(path: &str) -> Option<(&str, usize)> {
    if path.ends_with(']') {
        if let Some(start) = path.rfind('[') {
            if let Ok(idx) = path[start + 1..path.len() - 1].parse::<usize>() {
                return Some((&path[..start], idx));
            }
        }
    }
    None
}
