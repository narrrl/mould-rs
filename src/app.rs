use crate::format::EnvVar;
use tui_input::Input;

pub enum Mode {
    Normal,
    Insert,
}

pub struct App {
    pub vars: Vec<EnvVar>,
    pub selected: usize,
    pub mode: Mode,
    pub running: bool,
    pub status_message: Option<String>,
    pub input: Input,
}

impl App {
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

    pub fn next(&mut self) {
        if !self.vars.is_empty() {
            self.selected = (self.selected + 1) % self.vars.len();
            self.sync_input_with_selected();
        }
    }

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

    pub fn sync_input_with_selected(&mut self) {
        if let Some(var) = self.vars.get(self.selected) {
            self.input = Input::new(var.value.clone());
        }
    }

    pub fn commit_input(&mut self) {
        if let Some(var) = self.vars.get_mut(self.selected) {
            var.value = self.input.value().to_string();
        }
    }

    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
        self.status_message = None;
    }

    pub fn enter_normal(&mut self) {
        self.commit_input();
        self.mode = Mode::Normal;
    }

    #[allow(dead_code)]
    pub fn quit(&mut self) {
        self.running = false;
    }
}
