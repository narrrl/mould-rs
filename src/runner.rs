use crate::app::{App, InsertVariant, Mode};
use crate::config::Config;
use crate::format::FormatHandler;
use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::Terminal;
use ratatui::backend::Backend;
use std::io;
use std::path::Path;
use tui_input::backend::crossterm::EventHandler;

/// Manages the main application execution loop, event handling, and terminal interaction.
pub struct AppRunner<'a, B: Backend> {
    /// Reference to the terminal instance.
    terminal: &'a mut Terminal<B>,
    /// Mutable reference to the application state.
    app: &'a mut App,
    /// Loaded user configuration.
    config: &'a Config,
    /// Path where the final configuration will be saved.
    output_path: &'a Path,
    /// Handler for the specific file format (env, json, yaml, toml).
    handler: &'a dyn FormatHandler,
    /// Buffer for storing active command entry (e.g., ":w").
    command_buffer: String,
    /// Buffer for storing sequence of key presses (e.g., "gg").
    key_sequence: String,
}

impl<'a, B: Backend> AppRunner<'a, B>
where
    io::Error: From<B::Error>,
{
    /// Creates a new runner instance.
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
            key_sequence: String::new(),
        }
    }

    /// Starts the main application loop.
    pub fn run(&mut self) -> io::Result<()> {
        while self.app.running {
            self.terminal
                .draw(|f| crate::ui::draw(f, self.app, self.config))?;

            if let Event::Key(key) = event::read()? {
                self.handle_key_event(key)?;
            }
        }
        Ok(())
    }

    /// Primary dispatcher for all keyboard events.
    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.app.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::InsertKey => self.handle_insert_key_mode(key),
            Mode::Search => self.handle_search_mode(key),
        }
    }

    /// Handles keys in Normal mode, separating navigation from command entry.
    fn handle_normal_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        if !self.command_buffer.is_empty() {
            self.handle_command_mode(key)
        } else {
            self.handle_navigation_mode(key)
        }
    }

    /// Logic for entering and executing ":" style commands.
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

    /// Handles primary navigation (j/k) and transitions to insert or command modes.
    fn handle_navigation_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        let key_str = if let KeyCode::Char(c) = key.code {
            let mut s = String::new();
            if key.modifiers.contains(event::KeyModifiers::ALT) {
                s.push_str("alt+");
            }
            s.push(c);
            s
        } else {
            String::new()
        };

        if !key_str.is_empty() {
            self.key_sequence.push_str(&key_str);

            // Collect all configured keybinds
            let binds = [
                (&self.config.keybinds.down, "down"),
                (&self.config.keybinds.up, "up"),
                (&self.config.keybinds.edit, "edit"),
                (&self.config.keybinds.edit_append, "edit_append"),
                (&self.config.keybinds.edit_substitute, "edit_substitute"),
                (&self.config.keybinds.search, "search"),
                (&self.config.keybinds.next_match, "next_match"),
                (&self.config.keybinds.previous_match, "previous_match"),
                (&self.config.keybinds.jump_top, "jump_top"),
                (&self.config.keybinds.jump_bottom, "jump_bottom"),
                (&self.config.keybinds.append_item, "append_item"),
                (&self.config.keybinds.prepend_item, "prepend_item"),
                (&self.config.keybinds.delete_item, "delete_item"),
                (&self.config.keybinds.undo, "undo"),
                (&self.config.keybinds.redo, "redo"),
                (&self.config.keybinds.rename, "rename"),
                (&self.config.keybinds.append_group, "append_group"),
                (&self.config.keybinds.prepend_group, "prepend_group"),
                (&self.config.keybinds.toggle_group, "toggle_group"),
                (&"a".to_string(), "add_missing"),
                (&":".to_string(), "command"),
                (&"q".to_string(), "quit"),
            ];

            let mut exact_match = None;
            let mut prefix_match = false;

            for (bind, action) in binds.iter() {
                if bind == &&self.key_sequence {
                    exact_match = Some(*action);
                    break;
                } else if bind.starts_with(&self.key_sequence) {
                    prefix_match = true;
                }
            }

            if let Some(action) = exact_match {
                self.key_sequence.clear();
                match action {
                    "down" => self.app.next(),
                    "up" => self.app.previous(),
                    "edit" => self.app.enter_insert(InsertVariant::Start),
                    "edit_append" => self.app.enter_insert(InsertVariant::End),
                    "edit_substitute" => self.app.enter_insert(InsertVariant::Substitute),
                    "search" => {
                        self.app.mode = Mode::Search;
                        self.app.search_query.clear();
                        self.app.status_message = Some(format!("{} ", self.config.keybinds.search));
                    }
                    "next_match" => self.app.jump_next_match(),
                    "previous_match" => self.app.jump_previous_match(),
                    "jump_top" => self.app.jump_top(),
                    "jump_bottom" => self.app.jump_bottom(),
                    "append_item" => self.app.add_item(true, false, false),
                    "prepend_item" => self.app.add_item(false, false, false),
                    "delete_item" => self.app.delete_selected(),
                    "undo" => self.app.undo(),
                    "redo" => self.app.redo(),
                    "rename" => self.app.enter_insert_key(),
                    "append_group" => self.app.add_item(true, true, true),
                    "prepend_group" => self.app.add_item(false, true, true),
                    "toggle_group" => {
                        self.app.toggle_group_selected();
                        self.app.save_undo_state();
                    }
                    "add_missing" => {
                        self.add_missing_item();
                    }
                    "command" => {
                        self.command_buffer.push(':');
                        self.sync_command_status();
                    }
                    "quit" => self.app.running = false,
                    _ => {}
                }
            } else if !prefix_match {
                self.key_sequence.clear();
                self.key_sequence.push_str(&key_str);
            }
        } else {
            // Non-character keys reset the sequence buffer
            self.key_sequence.clear();
            match key.code {
                KeyCode::Down => self.app.next(),
                KeyCode::Up => self.app.previous(),
                KeyCode::Enter => self.save_file()?,
                KeyCode::Esc => self.app.status_message = None,
                _ => {}
            }
        }
        Ok(())
    }

    /// Adds a missing item from the template to the active configuration.
    fn add_missing_item(&mut self) {
        if let Some(var) = self.app.vars.get_mut(self.app.selected)
            && var.status == crate::format::ItemStatus::MissingFromActive {
                var.status = crate::format::ItemStatus::Present;
                if !var.is_group {
                    var.value = var.template_value.clone();
                }
                self.app.sync_input_with_selected();
                self.app.save_undo_state();
            }
    }

    /// Delegates key events to the `tui_input` handler during active editing.
    fn handle_insert_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.app.cancel_insert();
            }
            KeyCode::Enter => {
                self.app.enter_normal();
            }
            _ => {
                self.app.input.handle_event(&Event::Key(key));
            }
        }
        Ok(())
    }

    /// Handles keys in InsertKey mode.
    fn handle_insert_key_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.app.cancel_insert();
            }
            KeyCode::Enter => {
                self.app.enter_normal();
            }
            _ => {
                self.app.input.handle_event(&Event::Key(key));
            }
        }
        Ok(())
    }

    /// Handles search mode key events.
    fn handle_search_mode(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => {
                self.app.mode = Mode::Normal;
                self.app.status_message = None;
            }
            KeyCode::Backspace => {
                self.app.search_query.pop();
                self.app.status_message = Some(format!("{}{}", self.config.keybinds.search, self.app.search_query));
                self.app.jump_next_match();
            }
            KeyCode::Char(c) => {
                self.app.search_query.push(c);
                self.app.status_message = Some(format!("{}{}", self.config.keybinds.search, self.app.search_query));
                self.app.jump_next_match();
            }
            _ => {}
        }
        Ok(())
    }

    /// Logic to map command strings (like ":w") to internal application actions.
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

    /// Attempts to write the current app state to the specified output file.
    fn save_file(&mut self) -> io::Result<()> {
        if self.handler.write(self.output_path, &self.app.vars).is_ok() {
            self.app.status_message = Some(format!("Saved to {}", self.output_path.display()));
        } else {
            self.app.status_message = Some("Error saving file".to_string());
        }
        Ok(())
    }

    /// Synchronizes the status bar display with the active command buffer.
    fn sync_command_status(&mut self) {
        if self.command_buffer.is_empty() {
            self.app.status_message = None;
        } else {
            self.app.status_message = Some(self.command_buffer.clone());
        }
    }
}
