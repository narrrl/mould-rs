mod app;
mod cli;
mod config;
mod error;
mod format;
mod runner;
mod ui;
mod resolver;

use app::App;
use config::load_config;
use error::MouldError;
use format::{detect_format, get_handler};
use log::{error, info, warn};
use runner::AppRunner;
use std::io;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

fn main() -> anyhow::Result<()> {
    let args = cli::parse();

    // Initialize logger with verbosity from CLI
    let log_level = match args.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .init();

    let input_path = match args.input {
        Some(path) => {
            if !path.exists() {
                error!("Input file not found: {}", path.display());
                return Err(MouldError::FileNotFound(path.display().to_string()).into());
            }
            path
        }
        None => match resolver::find_input_file() {
            Some(path) => {
                info!("Discovered template: {}", path.display());
                path
            }
            None => {
                error!("No template file provided and none discovered in current directory.");
                println!("Usage: mould <INPUT_FILE>");
                println!("Supported defaults: .env.example, compose.yml, docker-compose.yml, etc.");
                return Err(MouldError::FileNotFound("None".to_string()).into());
            }
        },
    };

    info!("Input: {}", input_path.display());

    let format_type = detect_format(&input_path, args.format);
    let handler = get_handler(format_type);

    // Smart Comparison Logic
    let (active_path, template_path) = resolver::resolve_paths(&input_path);

    let output_path = args
        .output
        .unwrap_or_else(|| active_path.clone().unwrap_or_else(|| resolver::determine_output_path(&input_path)));

    info!("Output: {}", output_path.display());

    // 1. Load active config if it exists
    let mut vars = if let Some(active) = &active_path {
        handler.parse(active).unwrap_or_default()
    } else {
        Vec::new()
    };

    // 2. Load template config and merge
    if let Some(template) = &template_path {
        info!("Comparing with template: {}", template.display());
        let template_vars = handler.parse(template).unwrap_or_default();
        if vars.is_empty() {
             vars = template_vars;
             // If we only have template, everything is missing from active initially
             for v in vars.iter_mut() {
                 v.status = crate::format::ItemStatus::MissingFromActive;
                 v.value = None;
             }
        } else {
             // Merge template into active
             handler.merge(template, &mut vars).unwrap_or_default();
        }
    } else if vars.is_empty() {
        // Fallback if no template and active is empty
         vars = handler.parse(&input_path)?;
    }

    if vars.is_empty() {
        warn!("No variables found.");
    }

    let config = load_config();
    let mut app = App::new(vars);

    // Terminal lifecycle
    enable_raw_mode()
        .map_err(|e| MouldError::Terminal(format!("Failed to enable raw mode: {}", e)))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| MouldError::Terminal(format!("Failed to enter alternate screen: {}", e)))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| MouldError::Terminal(format!("Failed to create terminal backend: {}", e)))?;

    let mut runner = AppRunner::new(
        &mut terminal,
        &mut app,
        &config,
        &output_path,
        handler.as_ref(),
    );
    let res = runner.run();

    // Restoration
    disable_raw_mode().ok();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .ok();
    terminal.show_cursor().ok();

    match res {
        Ok(_) => {
            info!("Successfully finished mould session.");
            Ok(())
        }
        Err(e) => {
            error!("Application error during run: {}", e);
            Err(e.into())
        }
    }
}