use crate::app::{App, Mode};
use crate::config::Config;
use crate::format::FormatHandler;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::backend::Backend;
use ratatui::Terminal;
use std::io;
use std::path::Path;
use tui_input::backend::crossterm::EventHandler;

pub struct AppRunner<'a, B: Backend> {
    terminal: &'a mut Terminal<B>,
    app: &'a mut App,
    config: &'a Config,
    output_path: &'a Path,
    handler: &'a dyn FormatHandler,
    command_buffer: String,
}

impl<'a, B: Backend> AppRunner<'a, B>
where
    io::Error: From<B::Error>,
{
    pub fn new(
        terminal: &'a mut Terminal<B>,
        app: &'a mut App,
        config: &'a Config,
        output_path: &'a Path,
        handler: &'a dyn FormatHandler,
    ) -> Self {
        Self {
            terminal,
            app,
            config,
            output_path,
            handler,
            command_buffer: String::new(),
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        while self.app.running {
            self.terminal.draw(|f| crate::ui::draw(f, self.app, self.config))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key)?;
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.app.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        if !self.command_buffer.is_empty() {
            self.handle_command_mode(key)
        } else {
            self.handle_navigation_mode(key)
        }
    }

    fn handle_command_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Enter => {
                let cmd = self.command_buffer.clone();
                self.command_buffer.clear();
                self.execute_command(&cmd)
            }
            KeyCode::Esc => {
                self.command_buffer.clear();
                self.app.status_message = None;
                Ok(())
            }
            KeyCode::Backspace => {
                self.command_buffer.pop();
                self.sync_command_status();
                Ok(())
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
                self.sync_command_status();
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn handle_navigation_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        if let KeyCode::Char(c) = key.code {
            let c_str = c.to_string();
            if c_str == self.config.keybinds.down {
                self.app.next();
            } else if c_str == self.config.keybinds.up {
                self.app.previous();
            } else if c_str == self.config.keybinds.edit {
                self.app.enter_insert();
            } else if c_str == ":" {
                self.command_buffer.push(':');
                self.sync_command_status();
            } else if c_str == "q" {
                self.app.running = false;
            }
        } else {
            match key.code {
                KeyCode::Down => self.app.next(),
                KeyCode::Up => self.app.previous(),
                KeyCode::Enter => self.save_file()?,
                _ => {}
            }
        }
        Ok(())
    }

    fn handle_insert_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.app.enter_normal();
            }
            _ => {
                self.app.input.handle_event(&Event::Key(key));
            }
        }
        Ok(())
    }

    fn execute_command(&mut self, cmd: &str) -> io::Result<()> {
        if cmd == self.config.keybinds.save {
            self.save_file()
        } else if cmd == self.config.keybinds.quit {
            self.app.running = false;
            Ok(())
        } else if cmd == ":wq" {
            self.save_file()?;
            self.app.running = false;
            Ok(())
        } else {
            self.app.status_message = Some(format!("Unknown command: {}", cmd));
            Ok(())
        }
    }

    fn save_file(&mut self) -> io::Result<()> {
        if self.handler.write(self.output_path, &self.app.vars).is_ok() {
            self.app.status_message = Some(format!("Saved to {}", self.output_path.display()));
        } else {
            self.app.status_message = Some("Error saving file".to_string());
        }
        Ok(())
    }

    fn sync_command_status(&mut self) {
        if self.command_buffer.is_empty() {
            self.app.status_message = None;
        } else {
            self.app.status_message = Some(self.command_buffer.clone());
        }
    }
}
