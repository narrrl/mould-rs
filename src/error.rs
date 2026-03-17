use std::io;
use thiserror::Error;

/// Custom error types for the mould application.
#[derive(Error, Debug)]
pub enum MouldError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Format error: {0}")]
    Format(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Terminal error: {0}")]
    Terminal(String),
}
