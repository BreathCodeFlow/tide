use std::io;
use thiserror::Error;

/// Custom error types for Tide
#[derive(Error, Debug)]
pub enum TideError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Task execution failed: {0}")]
    TaskFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Command not found: {0}")]
    CommandNotFound(String),
}
