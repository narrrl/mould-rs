use clap::Parser;
use std::path::PathBuf;

/// mould: A TUI tool to generate and edit configuration files (.env, json, yaml, toml)
#[derive(Parser, Debug)]
#[command(
    author, 
    version, 
    about = "mould: A TUI tool to generate and edit configuration files (.env, json, yaml, toml)",
    long_about = "mould allows you to interactively edit and generate configuration files using templates. It supports various formats including .env, JSON, YAML, and TOML. It features a modern TUI with Vim-inspired keybindings and out-of-the-box support for theming.",
    after_help = "EXAMPLES:\n  mould .env.example\n  mould docker-compose.yml\n  mould config.template.json -o config.json"
)]
pub struct Cli {
    /// The input template file (e.g., .env.example, config.json.template, docker-compose.yml)
    #[arg(required = true, value_name = "INPUT_FILE")]
    pub input: PathBuf,

    /// Optional output file. If not provided, it will be inferred.
    #[arg(short, long, value_name = "OUTPUT_FILE")]
    pub output: Option<PathBuf>,

    /// Override the format detection (env, json, yaml, toml)
    #[arg(short, long, value_name = "FORMAT", value_parser = ["env", "json", "yaml", "toml"])]
    pub format: Option<String>,

    /// Increase verbosity for logging (can be used multiple times)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Parses and returns the command-line arguments.
pub fn parse() -> Cli {
    Cli::parse()
}
