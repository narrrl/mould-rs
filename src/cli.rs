use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "TUI tool to generate and edit configuration files (.env, json, yaml, toml)")]
pub struct Cli {
    /// The input template file (e.g., .env.example, config.json.template, docker-compose.yml)
    pub input: PathBuf,

    /// Optional output file. If not provided, it will be inferred (e.g., .env.example -> .env, docker-compose.yml -> docker-compose.override.yml)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Override the format detection (env, json, yaml, toml)
    #[arg(short, long)]
    pub format: Option<String>,
}

pub fn parse() -> Cli {
    Cli::parse()
}
