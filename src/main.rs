mod app;
mod cli;
mod config;
mod format;
mod runner;
mod ui;

use app::App;
use config::load_config;
use format::{detect_format, get_handler};
use runner::AppRunner;
use std::error::Error;
use std::io;
use std::path::{Path, PathBuf};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Helper to automatically determine the output file path based on common naming conventions.
fn determine_output_path(input: &Path) -> PathBuf {
    let file_name = input.file_name().unwrap_or_default().to_string_lossy();
    
    // .env.example -> .env
    if file_name == ".env.example" {
        return input.with_file_name(".env");
    }
    
    // docker-compose.yml -> docker-compose.override.yml
    if file_name == "docker-compose.yml" {
        return input.with_file_name("docker-compose.override.yml");
    }
    if file_name == "docker-compose.yaml" {
        return input.with_file_name("docker-compose.override.yaml");
    }
    
    // config.example.json -> config.json
    if file_name.ends_with(".example.json") {
        return input.with_file_name(file_name.replace(".example.json", ".json"));
    }
    if file_name.ends_with(".template.json") {
        return input.with_file_name(file_name.replace(".template.json", ".json"));
    }
    
    // Fallback: append .out to the extension
    input.with_extension(format!(
        "{}.out",
        input.extension().unwrap_or_default().to_string_lossy()
    ))
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse CLI arguments
    let args = cli::parse();

    let input_path = args.input;
    if !input_path.exists() {
        println!("Input file does not exist: {}", input_path.display());
        return Ok(());
    }

    // Detect format and select appropriate handler
    let format_type = detect_format(&input_path, args.format);
    let handler = get_handler(format_type);

    // Determine where to save the result
    let output_path = args
        .output
        .unwrap_or_else(|| determine_output_path(&input_path));

    // Initial parsing of the template file
    let mut vars = handler.parse(&input_path).unwrap_or_else(|err| {
        println!("Error parsing input file: {}", err);
        vec![]
    });

    if vars.is_empty() {
        println!(
            "No variables found in {} or file could not be parsed.",
            input_path.display()
        );
        return Ok(());
    }

    // Merge values from an existing output file if it exists
    if let Err(e) = handler.merge(&output_path, &mut vars) {
        println!("Warning: Could not merge existing output file: {}", e);
    }

    // Load user configuration and initialize application state
    let config = load_config();
    let mut app = App::new(vars);

    // Initialize terminal into raw mode and enter alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Instantiate the runner and start the application loop
    let mut runner = AppRunner::new(&mut terminal, &mut app, &config, &output_path, handler.as_ref());
    let res = runner.run();

    // Clean up terminal state on exit
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
