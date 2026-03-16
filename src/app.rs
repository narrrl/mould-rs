use crate::env::EnvVar;

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
}

impl App {
    pub fn new(vars: Vec<EnvVar>) -> Self {
        Self {
            vars,
            selected: 0,
            mode: Mode::Normal,
            running: true,
            status_message: None,
        }
    }

    pub fn next(&mut self) {
        if !self.vars.is_empty() {
            self.selected = (self.selected + 1) % self.vars.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.vars.is_empty() {
            if self.selected == 0 {
                self.selected = self.vars.len() - 1;
            } else {
                self.selected -= 1;
            }
        }
    }

    pub fn enter_insert(&mut self) {
        self.mode = Mode::Insert;
    }

    pub fn enter_normal(&mut self) {
        self.mode = Mode::Normal;
    }

    #[allow(dead_code)]
    pub fn quit(&mut self) {
        self.running = false;
    }
}
