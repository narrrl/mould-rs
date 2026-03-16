use crate::format::EnvVar;
use tui_input::Input;

/// Represents the current operating mode of the application.
pub enum Mode {
    /// Standard navigation and command mode.
    Normal,
    /// Active text entry mode for modifying values.
    Insert,
}

/// The core application state, holding all configuration variables and UI status.
pub struct App {
    /// The list of configuration variables being edited.
    pub vars: Vec<EnvVar>,
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
}

impl App {
    /// Initializes a new application instance with the provided variables.
    pub fn new(vars: Vec<EnvVar>) -> Self {
        let initial_input = vars.get(0).map(|v| v.value.clone()).unwrap_or_default();
        Self {
            vars,
            selected: 0,
            mode: Mode::Normal,
            running: true,
            status_message: None,
            input: Input::new(initial_input),
        }
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

    /// Updates the input buffer to reflect the value of the currently selected variable.
    pub fn sync_input_with_selected(&mut self) {
        if let Some(var) = self.vars.get(self.selected) {
            self.input = Input::new(var.value.clone());
        }
    }

    /// Commits the current text in the input buffer back to the selected variable's value.
    pub fn commit_input(&mut self) {
        if let Some(var) = self.vars.get_mut(self.selected) {
            var.value = self.input.value().to_string();
        }
    }

    /// Transitions the application into Insert Mode.
    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
        self.status_message = None;
    }

    /// Commits the current input and transitions the application into Normal Mode.
    pub fn enter_normal(&mut self) {
        self.commit_input();
        self.mode = Mode::Normal;
    }
}
