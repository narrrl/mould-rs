mod app;
mod config;
mod env;
mod ui;

use app::{App, Mode};
use config::load_config;
use env::{merge_env, parse_env_example, write_env};
use std::error::Error;
use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};

fn main() -> Result<(), Box<dyn Error>> {
    let example_path = ".env.example";
    let env_path = ".env";

    // Load vars
    let mut vars = parse_env_example(example_path).unwrap_or_else(|_| vec![]);
    if vars.is_empty() {
        println!("No variables found in .env.example or file does not exist.");
        println!("Please run this tool in a directory with a valid .env.example file.");
        return Ok(());
    }

    // Merge existing .env if present
    let _ = merge_env(env_path, &mut vars);

    let config = load_config();
    let mut app = App::new(vars);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal, &mut app, &config, env_path);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    config: &config::Config,
    env_path: &str,
) -> io::Result<()>
where
    io::Error: From<B::Error>,
{
    // For handling commands like :w, :q
    let mut command_buffer = String::new();

    loop {
        terminal.draw(|f| ui::draw(f, app, config))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => {
                    if !command_buffer.is_empty() {
                        if key.code == KeyCode::Enter {
                            match command_buffer.as_str() {
                                ":w" => {
                                    if write_env(env_path, &app.vars).is_ok() {
                                        app.status_message = Some("Saved to .env".to_string());
                                    } else {
                                        app.status_message =
                                            Some("Error saving to .env".to_string());
                                    }
                                }
                                ":q" => return Ok(()),
                                ":wq" => {
                                    write_env(env_path, &app.vars)?;
                                    return Ok(());
                                }
                                _ => {
                                    app.status_message = Some("Unknown command".to_string());
                                }
                            }
                            command_buffer.clear();
                        } else if key.code == KeyCode::Esc {
                            command_buffer.clear();
                            app.status_message = None;
                        } else if key.code == KeyCode::Backspace {
                            command_buffer.pop();
                            if command_buffer.is_empty() {
                                app.status_message = None;
                            } else {
                                app.status_message = Some(command_buffer.clone());
                            }
                        } else if let KeyCode::Char(c) = key.code {
                            command_buffer.push(c);
                            app.status_message = Some(command_buffer.clone());
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('j') | KeyCode::Down => app.next(),
                            KeyCode::Char('k') | KeyCode::Up => app.previous(),
                            KeyCode::Char('i') => {
                                app.enter_insert();
                                app.status_message = None;
                            }
                            KeyCode::Char(':') => {
                                command_buffer.push(':');
                                app.status_message = Some(command_buffer.clone());
                            }
                            KeyCode::Enter => {
                                // Default action for Enter in Normal mode is save
                                if write_env(env_path, &app.vars).is_ok() {
                                    app.status_message = Some("Saved to .env".to_string());
                                } else {
                                    app.status_message = Some("Error saving to .env".to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Mode::Insert => match key.code {
                    KeyCode::Esc => {
                        app.enter_normal();
                    }
                    KeyCode::Char(c) => {
                        if let Some(var) = app.vars.get_mut(app.selected) {
                            var.value.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(var) = app.vars.get_mut(app.selected) {
                            var.value.pop();
                        }
                    }
                    KeyCode::Enter => {
                        app.enter_normal();
                    }
                    _ => {}
                },
            }
        }

        if !app.running {
            break;
        }
    }
    Ok(())
}
