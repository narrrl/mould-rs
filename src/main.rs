mod app;
mod cli;
mod config;
mod error;
mod format;
mod runner;
mod ui;

use app::App;
use config::load_config;
use error::MouldError;
use format::{detect_format, get_handler};
use log::{error, info, warn};
use runner::AppRunner;
use std::io;
use std::path::{Path, PathBuf};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

/// Helper to automatically determine the output file path based on common naming conventions.
fn determine_output_path(input: &Path) -> PathBuf {
    let file_name = input.file_name().unwrap_or_default().to_string_lossy();

    if file_name == ".env.example" {
        return input.with_file_name(".env");
    }

    if file_name == "docker-compose.yml" || file_name == "compose.yml" {
        return input.with_file_name("compose.override.yml");
    }
    if file_name == "docker-compose.yaml" || file_name == "compose.yaml" {
        return input.with_file_name("compose.override.yaml");
    }

    if file_name.ends_with(".example.json") {
        return input.with_file_name(file_name.replace(".example.json", ".json"));
    }
    if file_name.ends_with(".template.json") {
        return input.with_file_name(file_name.replace(".template.json", ".json"));
    }

    input.with_extension(format!(
        "{}.out",
        input.extension().unwrap_or_default().to_string_lossy()
    ))
}

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

    let input_path = args.input;
    if !input_path.exists() {
        error!("Input file not found: {}", input_path.display());
        return Err(MouldError::FileNotFound(input_path.display().to_string()).into());
    }

    info!("Input: {}", input_path.display());

    let format_type = detect_format(&input_path, args.format);
    let handler = get_handler(format_type);

    let output_path = args
        .output
        .unwrap_or_else(|| determine_output_path(&input_path));

    info!("Output: {}", output_path.display());

    let mut vars = handler.parse(&input_path).map_err(|e| {
        error!("Failed to parse input file: {}", e);
        MouldError::Format(format!("Failed to parse {}: {}", input_path.display(), e))
    })?;

    if vars.is_empty() {
        warn!("No variables found in {}", input_path.display());
    }

    if let Err(e) = handler.merge(&output_path, &mut vars) {
        warn!("Could not merge existing output file: {}", e);
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
