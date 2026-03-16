use clap::Parser;
use std::path::PathBuf;

/// mould: A TUI tool to generate and edit configuration files (.env, json, yaml, toml)
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// The input template file (e.g., .env.example, config.json.template, docker-compose.yml)
    pub input: PathBuf,

    /// Optional output file. If not provided, it will be inferred.
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Override the format detection (env, json, yaml, toml)
    #[arg(short, long)]
    pub format: Option<String>,
}

/// Parses and returns the command-line arguments.
pub fn parse() -> Cli {
    Cli::parse()
}
