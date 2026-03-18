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

pub enum InsertVariant {
    Start,
    End,
    Substitute,
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
    /// Stack of previous variable states for undo functionality.
    pub undo_stack: Vec<Vec<ConfigItem>>,
}

impl App {
    /// Initializes a new application instance with the provided variables.
    pub fn new(vars: Vec<ConfigItem>) -> Self {
        let initial_input = vars.first().and_then(|v| v.value.clone()).unwrap_or_default();
        Self {
            vars,
            selected: 0,
            mode: Mode::Normal,
            running: true,
            status_message: None,
            input: Input::new(initial_input),
            search_query: String::new(),
            undo_stack: Vec::new(),
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
        if let Some(var) = self.vars.get_mut(self.selected)
            && !var.is_group {
                var.value = Some(self.input.value().to_string());
                var.status = crate::format::ItemStatus::Modified;
            }
    }

    /// Transitions the application into Insert Mode with a specific variant.
    pub fn enter_insert(&mut self, variant: InsertVariant) {
        if let Some(var) = self.vars.get(self.selected)
            && !var.is_group {
                self.save_undo_state();
                self.mode = Mode::Insert;
                match variant {
                    InsertVariant::Start => {
                        use tui_input::InputRequest;
                        self.input.handle(InputRequest::GoToStart);
                    }
                    InsertVariant::End => {
                        use tui_input::InputRequest;
                        self.input.handle(InputRequest::GoToEnd);
                    }
                    InsertVariant::Substitute => {
                        self.input = Input::new(String::new());
                    }
                }
            }
    }

    /// Commits the current input and transitions the application into Normal Mode.
    pub fn enter_normal(&mut self) {
        self.commit_input();
        self.mode = Mode::Normal;
    }

    /// Deletes the currently selected item. If it's a group, deletes all children.
    pub fn delete_selected(&mut self) {
        if self.vars.is_empty() {
            return;
        }

        self.save_undo_state();
        let selected_path = self.vars[self.selected].path.clone();
        let is_group = self.vars[self.selected].is_group;

        // Identify if the item being removed is an array item
        let array_info = parse_index(&selected_path);

        // 1. Identify all items to remove
        let mut to_remove = Vec::new();
        to_remove.push(self.selected);

        if is_group {
            let prefix_dot = format!("{}.", selected_path);
            let prefix_bracket = format!("{}[", selected_path);
            for (i, var) in self.vars.iter().enumerate() {
                if i == self.selected {
                    continue;
                }
                if var.path.starts_with(&prefix_dot) || var.path.starts_with(&prefix_bracket) {
                    to_remove.push(i);
                }
            }
        }

        // 2. Perform removal (reverse order to preserve indices)
        to_remove.sort_unstable_by(|a, b| b.cmp(a));
        for i in to_remove {
            self.vars.remove(i);
        }

        // 3. Re-index subsequent array items if applicable
        if let Some((base, removed_idx)) = array_info {
            let base = base.to_string();
            for var in self.vars.iter_mut() {
                if var.path.starts_with(&base) {
                    // We need to find the index segment that matches this array
                    if let Some((b, i, suffix)) = find_array_segment(&var.path, &base)
                        && b == base && i > removed_idx {
                            let new_idx = i - 1;
                            var.path = format!("{}[{}]{}", base, new_idx, suffix);
                            // Also update key if it matches the old index exactly
                            if var.key == format!("[{}]", i) {
                                var.key = format!("[{}]", new_idx);
                            }
                        }
                }
            }
        }

        // 4. Adjust selection
        if self.selected >= self.vars.len() && !self.vars.is_empty() {
            self.selected = self.vars.len() - 1;
        }
        self.sync_input_with_selected();
    }

    /// Adds a new item to an array if the selected item is part of one.
    pub fn add_array_item(&mut self, after: bool) {
        if self.vars.is_empty() {
            return;
        }

        self.save_undo_state();
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
            if var.path.starts_with(&base)
                && let Some((b, i)) = parse_index(&var.path)
                    && b == base && i >= new_idx {
                        var.path = format!("{}[{}]", base, i + 1);
                        // Also update key if it was just the index
                        if var.key == format!("[{}]", i) {
                            var.key = format!("[{}]", i + 1);
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
        self.enter_insert(InsertVariant::Start);
        self.status_message = None;
    }

    /// Status bar helpers
    pub fn selected_is_group(&self) -> bool {
        self.vars.get(self.selected).map(|v| v.is_group).unwrap_or(false)
    }

    pub fn selected_is_array(&self) -> bool {
        self.vars.get(self.selected)
            .map(|v| !v.is_group && v.path.contains('['))
            .unwrap_or(false)
    }

    pub fn selected_is_missing(&self) -> bool {
        self.vars.get(self.selected)
            .map(|v| v.status == crate::format::ItemStatus::MissingFromActive)
            .unwrap_or(false)
    }

    /// Saves the current state of variables to the undo stack.
    pub fn save_undo_state(&mut self) {
        self.undo_stack.push(self.vars.clone());
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    /// Reverts to the last saved state of variables.
    pub fn undo(&mut self) {
        if let Some(previous_vars) = self.undo_stack.pop() {
            self.vars = previous_vars;
            if self.selected >= self.vars.len() && !self.vars.is_empty() {
                self.selected = self.vars.len() - 1;
            }
            self.sync_input_with_selected();
            self.status_message = Some("Undo applied".to_string());
        } else {
            self.status_message = Some("Nothing to undo".to_string());
        }
    }
}

fn parse_index(path: &str) -> Option<(&str, usize)> {
    if let Some(end) = path.rfind(']') {
        let segment = &path[..=end];
        if let Some(start) = segment.rfind('[')
            && let Ok(idx) = segment[start + 1..end].parse::<usize>() {
                // Return the base and index
                return Some((&path[..start], idx));
            }
    }
    None
}

/// Helper to find an array segment in a path given a base prefix.
fn find_array_segment<'a>(path: &'a str, base: &str) -> Option<(&'a str, usize, &'a str)> {
    if !path.starts_with(base) {
        return None;
    }
    let remaining = &path[base.len()..];
    if remaining.starts_with('[')
        && let Some(end) = remaining.find(']')
            && let Ok(idx) = remaining[1..end].parse::<usize>() {
                return Some((&path[..base.len()], idx, &remaining[end + 1..]));
            }
    None
}
