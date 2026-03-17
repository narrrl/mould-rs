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

    // Standard mappings
    if file_name == ".env.example" || file_name == ".env.template" {
        return input.with_file_name(".env");
    }

    if file_name == "docker-compose.yml" || file_name == "compose.yml" {
        return input.with_file_name("compose.override.yml");
    }
    if file_name == "docker-compose.yaml" || file_name == "compose.yaml" {
        return input.with_file_name("compose.override.yaml");
    }

    // Pattern-based mappings
    if let Some(base) = file_name.strip_suffix(".env.example") {
        return input.with_file_name(format!("{}.env", base));
    }
    if let Some(base) = file_name.strip_suffix(".env.template") {
        return input.with_file_name(format!("{}.env", base));
    }
    if let Some(base) = file_name.strip_suffix(".example.json") {
        return input.with_file_name(format!("{}.json", base));
    }
    if let Some(base) = file_name.strip_suffix(".template.json") {
        return input.with_file_name(format!("{}.json", base));
    }
    if let Some(base) = file_name.strip_suffix(".example.yml") {
        return input.with_file_name(format!("{}.yml", base));
    }
    if let Some(base) = file_name.strip_suffix(".template.yml") {
        return input.with_file_name(format!("{}.yml", base));
    }
    if let Some(base) = file_name.strip_suffix(".example.yaml") {
        return input.with_file_name(format!("{}.yaml", base));
    }
    if let Some(base) = file_name.strip_suffix(".template.yaml") {
        return input.with_file_name(format!("{}.yaml", base));
    }
    if let Some(base) = file_name.strip_suffix(".example.toml") {
        return input.with_file_name(format!("{}.toml", base));
    }
    if let Some(base) = file_name.strip_suffix(".template.toml") {
        return input.with_file_name(format!("{}.toml", base));
    }

    input.with_extension(format!(
        "{}.out",
        input.extension().unwrap_or_default().to_string_lossy()
    ))
}

/// Discovers common configuration template files in the current directory.
fn find_input_file() -> Option<PathBuf> {
    let candidates = [
        ".env.example",
        "compose.yml",
        "docker-compose.yml",
        ".env.template",
        "compose.yaml",
        "docker-compose.yaml",
    ];

    // Priority 1: Exact matches for well-known defaults
    for name in &candidates {
        let path = PathBuf::from(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Priority 2: Pattern matches
    if let Ok(entries) = std::fs::read_dir(".") {
        let mut fallback = None;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.ends_with(".env.example")
                || name_str.ends_with(".env.template")
                || name_str.ends_with(".example.json")
                || name_str.ends_with(".template.json")
                || name_str.ends_with(".example.yml")
                || name_str.ends_with(".template.yml")
                || name_str.ends_with(".example.yaml")
                || name_str.ends_with(".template.yaml")
                || name_str.ends_with(".example.toml")
                || name_str.ends_with(".template.toml")
            {
                // Prefer .env.* or compose.* if multiple matches
                if name_str.contains(".env") || name_str.contains("compose") {
                    return Some(entry.path());
                }
                if fallback.is_none() {
                    fallback = Some(entry.path());
                }
            }
        }
        if let Some(path) = fallback {
            return Some(path);
        }
    }

    None
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

    let input_path = match args.input {
        Some(path) => {
            if !path.exists() {
                error!("Input file not found: {}", path.display());
                return Err(MouldError::FileNotFound(path.display().to_string()).into());
            }
            path
        }
        None => match find_input_file() {
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
    let input_name = input_path.file_name().unwrap_or_default().to_string_lossy();
    let is_template_input = input_name.contains(".example") || input_name.contains(".template") || input_name == "compose.yml" || input_name == "docker-compose.yml";
    
    let mut template_path = None;
    let mut active_path = None;

    if is_template_input {
        template_path = Some(input_path.clone());
        let expected_active = determine_output_path(&input_path);
        if expected_active.exists() {
            active_path = Some(expected_active);
        }
    } else {
        // Input is likely an active config (e.g., .env)
        active_path = Some(input_path.clone());
        // Try to find a template
        let possible_templates = [
            format!("{}.example", input_name),
            format!("{}.template", input_name),
        ];
        for t in possible_templates {
            let p = input_path.with_file_name(t);
            if p.exists() {
                template_path = Some(p);
                break;
            }
        }
    }

    let output_path = args
        .output
        .unwrap_or_else(|| active_path.clone().unwrap_or_else(|| determine_output_path(&input_path)));

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
         vars = handler.parse(&input_path).map_err(|e| {
            error!("Failed to parse input file: {}", e);
            MouldError::Format(format!("Failed to parse {}: {}", input_path.display(), e))
        })?;
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
