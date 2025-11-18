use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/// Simple thread-safe file logger for task execution traces.
pub struct Logger {
    file: Mutex<File>,
}

impl Logger {
    /// Create (or append to) the log file at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create log directory {}", parent.display()))?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .with_context(|| format!("Failed to open log file {}", path.display()))?;

        Ok(Self {
            file: Mutex::new(file),
        })
    }

    /// Write a single log line with a timestamp prefix.
    pub fn log_line(&self, message: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let mut guard = self
            .file
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock log file"))?;
        writeln!(guard, "[{}] {}", timestamp, message)?;
        Ok(())
    }

    /// Write a message followed by an indented multiline block.
    pub fn log_block(&self, header: &str, body: &str) -> Result<()> {
        self.log_line(header)?;
        for line in body.lines() {
            let indent = format!("    {}", line);
            self.log_line(&indent)?;
        }
        Ok(())
    }
}
