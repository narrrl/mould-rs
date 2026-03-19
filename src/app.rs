use crate::format::{ConfigItem, PathSegment};
use tui_input::Input;
use crate::undo::UndoTree;

/// Represents the current operating mode of the application.
/// 
/// Modality allows the application to reuse the same keyboard events 
/// for different contextual actions (navigation vs. text entry).
pub enum Mode {
    /// Standard navigation and command mode.
    Normal,
    /// Active text entry mode for modifying values.
    Insert,
    /// Active text entry mode for modifying keys.
    InsertKey,
    /// Active search mode for filtering keys.
    Search,
}

/// Defines where the cursor starts when entering Insert mode.
pub enum InsertVariant {
    /// Cursor at the beginning of the text.
    Start,
    /// Cursor at the end of the text.
    End,
    /// Text is cleared before entry.
    Substitute,
}

/// The core application state, holding all configuration variables and UI status.
///
/// This struct is the "Single Source of Truth" for the TUI. It manages 
/// selection, filtering, history, and structural mutations.
pub struct App {
    /// The flattened list of configuration variables being edited.
    pub vars: Vec<ConfigItem>,
    /// Index of the currently selected variable in the list.
    pub selected: usize,
    /// The current interaction mode (Normal, Insert, etc.).
    pub mode: Mode,
    /// Whether the main application loop should continue running.
    pub running: bool,
    /// An optional message to display in the status bar.
    pub status_message: Option<String>,
    /// The active text input buffer for the selected variable.
    pub input: Input,
    /// The current search query for filtering keys.
    pub search_query: String,
    /// Undo history structured as a tree.
    pub undo_tree: UndoTree,
}

impl App {
    /// Initializes a new application instance with the provided variables.
    ///
    /// It automatically initializes the undo tree with the starting state.
    pub fn new(vars: Vec<ConfigItem>) -> Self {
        let initial_input = vars.first().and_then(|v| v.value.clone()).unwrap_or_default();
        let undo_tree = UndoTree::new(vars.clone(), 0);
        Self {
            vars,
            selected: 0,
            mode: Mode::Normal,
            running: true,
            status_message: None,
            input: Input::new(initial_input),
            search_query: String::new(),
            undo_tree,
        }
    }

    /// Returns the indices of variables that match the search query (case-insensitive).
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

    /// Jumps the selection to the top of the list.
    pub fn jump_top(&mut self) {
        if !self.vars.is_empty() {
            self.selected = 0;
            self.sync_input_with_selected();
        }
    }

    /// Jumps the selection to the bottom of the list.
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

    /// Updates the input buffer to reflect the current state of the selected item.
    ///
    /// If in `InsertKey` mode, the buffer is synced with the item's `key`.
    /// Otherwise, it is synced with the item's `value`.
    pub fn sync_input_with_selected(&mut self) {
        if let Some(var) = self.vars.get(self.selected) {
            let val = match self.mode {
                Mode::InsertKey => var.key.clone(),
                _ => var.value.clone().unwrap_or_default(),
            };
            self.input = Input::new(val);
        }
    }

    /// Commits the current text in the input buffer back to the selected variable.
    /// 
    /// Returns true if commit was successful, false if there was an error 
    /// (e.g., a key name collision or empty key).
    pub fn commit_input(&mut self) -> bool {
        match self.mode {
            Mode::Insert => {
                if let Some(var) = self.vars.get_mut(self.selected)
                    && !var.is_group {
                        var.value = Some(self.input.value().to_string());
                        var.status = crate::format::ItemStatus::Modified;
                    }
                true
            }
            Mode::InsertKey => {
                let new_key = self.input.value().trim().to_string();
                if new_key.is_empty() {
                    self.status_message = Some("Key cannot be empty".to_string());
                    return false;
                }
                
                let selected_var = self.vars[self.selected].clone();
                if selected_var.key == new_key {
                    return true;
                }
                
                // Collision check: ensure siblings don't already have this key.
                let parent_path = if selected_var.path.len() > 1 {
                    &selected_var.path[..selected_var.path.len() - 1]
                } else {
                    &[]
                };
                
                let exists = self.vars.iter().enumerate().any(|(i, v)| {
                    i != self.selected 
                    && v.path.len() == selected_var.path.len() 
                    && v.path.starts_with(parent_path)
                    && v.key == new_key
                });
                
                if exists {
                    self.status_message = Some(format!("Key already exists: {}", new_key));
                    return false;
                }
                
                // Update selected item's key and its full internal path.
                let old_path = selected_var.path.clone();
                let mut new_path = parent_path.to_vec();
                new_path.push(PathSegment::Key(new_key.clone()));
                
                {
                    let var = self.vars.get_mut(self.selected).unwrap();
                    var.key = new_key;
                    var.path = new_path.clone();
                    var.status = crate::format::ItemStatus::Modified;
                }
                
                // Recursively update paths of all children if the renamed item is a group.
                if selected_var.is_group {
                    for var in self.vars.iter_mut() {
                        if var.path.starts_with(&old_path) && var.path.len() > old_path.len() {
                            let mut p = new_path.clone();
                            p.extend(var.path[old_path.len()..].iter().cloned());
                            var.path = p;
                            var.status = crate::format::ItemStatus::Modified;
                        }
                    }
                }
                true
            }
            _ => true,
        }
    }

    /// Transitions the application into `InsertKey` mode to modify item names.
    ///
    /// Renaming is blocked for array indices (e.g., `[0]`) as they are 
    /// managed automatically by the application logic.
    pub fn enter_insert_key(&mut self) {
        if !self.vars.is_empty() {
            if let Some(var) = self.vars.get(self.selected)
                && matches!(var.path.last(), Some(PathSegment::Index(_))) {
                    self.status_message = Some("Cannot rename array indices".to_string());
                    return;
                }
            self.mode = Mode::InsertKey;
            self.sync_input_with_selected();
        }
    }

    /// Transitions the application into `Insert` mode to modify variable values.
    ///
    /// If the selected item is a group, it automatically routes to 
    /// `enter_insert_key` instead.
    pub fn enter_insert(&mut self, variant: InsertVariant) {
        if let Some(var) = self.vars.get(self.selected) {
            if var.is_group {
                self.enter_insert_key();
            } else {
                if !matches!(variant, InsertVariant::Substitute) {
                    self.sync_input_with_selected();
                }
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
    }

    /// Commits the current input and transitions back to `Normal` mode.
    pub fn enter_normal(&mut self) {
        if self.commit_input() {
            self.save_undo_state();
            self.mode = Mode::Normal;
        }
    }

    /// Aborts the current input and reverts to `Normal` mode without saving changes.
    pub fn cancel_insert(&mut self) {
        self.mode = Mode::Normal;
        self.sync_input_with_selected();
        self.status_message = None;
    }

    /// Deletes the currently selected item and all its nested children.
    ///
    /// If the deleted item is part of an array, subsequent indices are 
    /// automatically shifted and renamed to maintain a continuous sequence.
    pub fn delete_selected(&mut self) {
        if self.vars.is_empty() {
            return;
        }

        let selected_path = self.vars[self.selected].path.clone();
        let is_group = self.vars[self.selected].is_group;

        // 1. Identify all items to remove (the item itself + all children)
        let mut to_remove = Vec::new();
        to_remove.push(self.selected);

        if is_group {
            for (i, var) in self.vars.iter().enumerate() {
                if i == self.selected {
                    continue;
                }
                if var.path.starts_with(&selected_path) {
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
        if let Some(PathSegment::Index(removed_idx)) = selected_path.last() {
            let base_path = &selected_path[..selected_path.len() - 1];
            
            for var in self.vars.iter_mut() {
                if var.path.starts_with(base_path) && var.path.len() >= selected_path.len() {
                    if let PathSegment::Index(i) = var.path[selected_path.len() - 1]
                        && i > *removed_idx {
                            let new_idx = i - 1;
                            var.path[selected_path.len() - 1] = PathSegment::Index(new_idx);
                            
                            if var.path.len() == selected_path.len() {
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
        self.save_undo_state();
    }

    /// Adds a new item relative to the selected item.
    ///
    /// - `after`: If true, adds below the selection; otherwise adds above.
    /// - `is_group`: If true, creates a new structural node (object/array).
    /// - `as_child`: If true, adds inside the selected group.
    ///
    /// The method automatically detects if the parent is an array and 
    /// formats the new key accordingly (e.g., `[1]`).
    pub fn add_item(&mut self, after: bool, is_group: bool, as_child: bool) {
        if self.vars.is_empty() {
            let new_key = if is_group { "NEW_GROUP".to_string() } else { "NEW_VAR".to_string() };
            self.vars.push(ConfigItem {
                key: new_key.clone(),
                path: vec![PathSegment::Key(new_key)],
                value: if is_group { None } else { Some("".to_string()) },
                template_value: None,
                default_value: None,
                depth: 0,
                is_group,
                status: crate::format::ItemStatus::Modified,
                value_type: if is_group { crate::format::ValueType::Null } else { crate::format::ValueType::String },
            });
            self.selected = 0;
            self.sync_input_with_selected();
            self.save_undo_state();
            if is_group {
                self.enter_insert_key();
            } else {
                self.enter_insert(InsertVariant::Start);
            }
            return;
        }

        let selected_item = self.vars[self.selected].clone();
        
        // 1. Determine new item properties (path, key, depth, position)
        let mut new_path;
        let new_depth;
        let insert_pos;
        let mut is_array_item = false;

        if !as_child && let Some(PathSegment::Index(idx)) = selected_item.path.last() {
            // ARRAY ITEM LOGIC (Adding sibling to an existing index)
            is_array_item = true;
            let base_path = selected_item.path[..selected_item.path.len() - 1].to_vec();
            let new_idx = if after { idx + 1 } else { *idx };
            insert_pos = if after { self.selected + 1 } else { self.selected };
            
            // Shift subsequent indices
            for var in self.vars.iter_mut() {
                if var.path.starts_with(&base_path) && var.path.len() > base_path.len()
                    && let PathSegment::Index(i) = var.path[base_path.len()]
                        && i >= new_idx {
                            var.path[base_path.len()] = PathSegment::Index(i + 1);
                            if var.path.len() == base_path.len() + 1 {
                                var.key = format!("[{}]", i + 1);
                            }
                        }
            }
            
            new_path = base_path;
            new_path.push(PathSegment::Index(new_idx));
            new_depth = selected_item.depth;
        } else if as_child && selected_item.is_group {
            // ADD AS CHILD OF GROUP
            insert_pos = self.selected + 1;
            new_path = selected_item.path.clone();
            new_depth = selected_item.depth + 1;
            
            if self.is_array_group(&selected_item.path) {
                is_array_item = true;
                let new_idx = 0; // Prepend to array
                new_path.push(PathSegment::Index(new_idx));
                
                // Shift existing children
                for var in self.vars.iter_mut() {
                    if var.path.starts_with(&selected_item.path) && var.path.len() > selected_item.path.len()
                        && let PathSegment::Index(i) = var.path[selected_item.path.len()] {
                            var.path[selected_item.path.len()] = PathSegment::Index(i + 1);
                            if var.path.len() == selected_item.path.len() + 1 {
                                var.key = format!("[{}]", i + 1);
                            }
                        }
                }
            }
        } else {
            // ADD AS SIBLING
            let parent_path = if selected_item.path.len() > 1 {
                selected_item.path[..selected_item.path.len() - 1].to_vec()
            } else {
                Vec::new()
            };
            
            insert_pos = if after {
                let mut p = self.selected + 1;
                while p < self.vars.len() && self.vars[p].path.starts_with(&selected_item.path) {
                    p += 1;
                }
                p
            } else {
                self.selected
            };
            
            new_path = parent_path;
            new_depth = selected_item.depth;
            
            if !new_path.is_empty() && self.is_array_group(&new_path) {
                is_array_item = true;
                if let Some(PathSegment::Index(idx)) = selected_item.path.last() {
                    let new_idx = if after { idx + 1 } else { *idx };
                    new_path.push(PathSegment::Index(new_idx));
                } else {
                    new_path.push(PathSegment::Index(0));
                }
            }
        }

        // 2. Generate a unique key for non-array items
        let final_key = if is_array_item {
            if let Some(PathSegment::Index(idx)) = new_path.last() {
                format!("[{}]", idx)
            } else {
                "NEW_VAR".to_string()
            }
        } else {
            let mut count = 1;
            let mut candidate = if is_group { "NEW_GROUP".to_string() } else { "NEW_VAR".to_string() };
            let parent_path_slice = new_path.as_slice();
            
            while self.vars.iter().any(|v| {
                v.path.starts_with(parent_path_slice) 
                && v.path.len() == parent_path_slice.len() + 1 
                && v.key == candidate
            }) {
                candidate = if is_group { format!("NEW_GROUP_{}", count) } else { format!("NEW_VAR_{}", count) };
                count += 1;
            }
            new_path.push(PathSegment::Key(candidate.clone()));
            candidate
        };

        // 3. Insert new item
        let new_item = ConfigItem {
            key: final_key,
            path: new_path,
            value: if is_group { None } else { Some("".to_string()) },
            template_value: None,
            default_value: None,
            depth: new_depth,
            is_group,
            status: crate::format::ItemStatus::Modified,
            value_type: if is_group { crate::format::ValueType::Null } else { crate::format::ValueType::String },
        };

        self.vars.insert(insert_pos, new_item);
        self.selected = insert_pos;
        self.save_undo_state();
        
        if is_array_item {
            self.sync_input_with_selected();
            self.enter_insert(InsertVariant::Start);
        } else {
            self.enter_insert_key();
        }
        self.status_message = None;
    }

    /// Toggles the group status of the currently selected item.
    ///
    /// Changing a group to a variable clears its children (visually) 
    /// and resets its value. Changing a variable to a group removes its value.
    pub fn toggle_group_selected(&mut self) {
        if let Some(var) = self.vars.get_mut(self.selected) {
            if matches!(var.path.last(), Some(PathSegment::Index(_))) {
                self.status_message = Some("Cannot toggle array items".to_string());
                return;
            }

            var.is_group = !var.is_group;
            if var.is_group {
                var.value = None;
                var.value_type = crate::format::ValueType::Null;
            } else {
                var.value = Some("".to_string());
                var.value_type = crate::format::ValueType::String;
            }
            var.status = crate::format::ItemStatus::Modified;
            self.sync_input_with_selected();
        }
    }

    /// Returns true if the selected item is a structural node (group/object).
    pub fn selected_is_group(&self) -> bool {
        self.vars.get(self.selected).map(|v| v.is_group).unwrap_or(false)
    }

    /// Returns true if the provided path identifies a node that contains array elements.
    pub fn is_array_group(&self, group_path: &[PathSegment]) -> bool {
        self.vars.iter().any(|v| 
            v.path.starts_with(group_path) 
            && v.path.len() == group_path.len() + 1 
            && matches!(v.path.last(), Some(PathSegment::Index(_)))
        )
    }

    /// Returns true if the selected item is an indexed array element.
    pub fn selected_is_array(&self) -> bool {
        self.vars.get(self.selected)
            .map(|v| !v.is_group && matches!(v.path.last(), Some(PathSegment::Index(_))))
            .unwrap_or(false)
    }

    /// Returns true if the selected item exists in the template but not the active config.
    pub fn selected_is_missing(&self) -> bool {
        self.vars.get(self.selected)
            .map(|v| v.status == crate::format::ItemStatus::MissingFromActive)
            .unwrap_or(false)
    }

    /// Saves a snapshot of the current state to the undo history tree.
    pub fn save_undo_state(&mut self) {
        self.undo_tree.push(self.vars.clone(), self.selected);
    }

    /// Reverts the application state to the previous history point.
    pub fn undo(&mut self) {
        if let Some(action) = self.undo_tree.undo() {
            self.vars = action.state.clone();
            self.selected = action.selected;
            if self.selected >= self.vars.len() && !self.vars.is_empty() {
                self.selected = self.vars.len() - 1;
            }
            self.sync_input_with_selected();
            self.status_message = Some("Undo applied".to_string());
        } else {
            self.status_message = Some("Nothing to undo".to_string());
        }
    }

    /// Advances the application state to the next history point in the active branch.
    pub fn redo(&mut self) {
        if let Some(action) = self.undo_tree.redo() {
            self.vars = action.state.clone();
            self.selected = action.selected;
            if self.selected >= self.vars.len() && !self.vars.is_empty() {
                self.selected = self.vars.len() - 1;
            }
            self.sync_input_with_selected();
            self.status_message = Some("Redo applied".to_string());
        } else {
            self.status_message = Some("Nothing to redo".to_string());
        }
    }
}
