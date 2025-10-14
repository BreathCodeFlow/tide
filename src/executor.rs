use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Password};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use shellexpand;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::TaskConfig;
use crate::keychain;

/// Task execution result
#[derive(Debug)]
pub struct TaskResult {
    pub name: String,
    pub group: String,
    pub group_icon: String,
    pub status: TaskStatus,
    pub duration: Duration,
    pub output: Option<String>,
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Success,
    Failed,
    Skipped,
}

/// Task executor with progress tracking
#[derive(Clone)]
pub struct TaskExecutor {
    pub multi_progress: Arc<MultiProgress>,
    pub dry_run: bool,
    pub verbose: bool,
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(dry_run: bool, verbose: bool) -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            dry_run,
            verbose,
        }
    }

    /// Create a configured spinner progress bar
    pub fn new_spinner(&self) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::with_template("{spinner:.cyan} {msg}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        pb.enable_steady_tick(Duration::from_millis(120));
        pb
    }

    /// Execute a single task
    pub async fn execute_task(
        &self,
        task: TaskConfig,
        group_name: String,
        group_icon: String,
        pb: ProgressBar,
        keychain_label: &str,
    ) -> TaskResult {
        let start = Instant::now();
        let task_name = task.name.clone();
        let group_label = format_group_label(&group_name, &group_icon);
        let task_label = format_task_label(&task_name, &task.icon);
        let progress_label = format!("[{}] {}", group_label, task_label);

        // Update progress bar
        pb.set_message(format!(
            "{} {}",
            progress_label.clone().bold(),
            "Running…".bright_white()
        ));

        if self.dry_run {
            tokio::time::sleep(Duration::from_millis(100)).await;
            pb.finish_with_message(format!(
                "{} {} {}",
                progress_label.clone().bold(),
                "○".yellow(),
                "[dry run]".dimmed()
            ));
            return TaskResult {
                name: task_name.clone(),
                group: group_name,
                group_icon,
                status: TaskStatus::Skipped,
                duration: start.elapsed(),
                output: Some("Dry run - command not executed".to_string()),
            };
        }

        // Check preconditions
        if let Some(check_cmd) = &task.check_command {
            if !keychain::command_exists(check_cmd) {
                pb.finish_with_message(format!(
                    "{} {}",
                    progress_label.clone().bold(),
                    "[skipped: command not found]".dimmed()
                ));
                return TaskResult {
                    name: task_name.clone(),
                    group: group_name,
                    group_icon,
                    status: TaskStatus::Skipped,
                    duration: start.elapsed(),
                    output: Some(format!("Command '{}' not found", check_cmd)),
                };
            }
        }

        if let Some(check_path) = &task.check_path {
            let expanded = shellexpand::tilde(check_path);
            if !Path::new(expanded.as_ref()).exists() {
                pb.finish_with_message(format!(
                    "{} {}",
                    progress_label.clone().bold(),
                    "[skipped: path not found]".dimmed()
                ));
                return TaskResult {
                    name: task_name.clone(),
                    group: group_name,
                    group_icon,
                    status: TaskStatus::Skipped,
                    duration: start.elapsed(),
                    output: Some(format!("Path '{}' not found", check_path)),
                };
            }
        }

        // Build command
        let mut cmd = task.command.clone();
        if task.sudo && !cmd.is_empty() && cmd[0] != "sudo" {
            cmd.insert(0, "sudo".to_string());
        }

        // Execute command
        let result = if cmd.get(0).map(|s| s.as_str()) == Some("sudo") {
            self.run_sudo_command(&cmd[1..], keychain_label).await
        } else {
            self.run_command(&cmd, &task).await
        };

        let (status, output) = match result {
            Ok(output) => (TaskStatus::Success, Some(output)),
            Err(e) if task.required => (TaskStatus::Failed, Some(e.to_string())),
            Err(e) => (TaskStatus::Skipped, Some(e.to_string())),
        };

        let duration = start.elapsed();
        let status_icon = match status {
            TaskStatus::Success => "✓".green(),
            TaskStatus::Failed => "✗".red(),
            TaskStatus::Skipped => "○".yellow(),
        };

        pb.finish_with_message(format!(
            "{} {} {}",
            progress_label.bold(),
            status_icon,
            format!("({})", format_duration(duration)).dimmed()
        ));

        TaskResult {
            name: task_name,
            group: group_name,
            group_icon,
            status,
            duration,
            output,
        }
    }

    /// Run a regular command
    async fn run_command(&self, cmd: &[String], task: &TaskConfig) -> Result<String> {
        if cmd.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let mut command = Command::new(&cmd[0]);
        command.args(&cmd[1..]);

        // Set working directory if specified
        if let Some(dir) = &task.working_dir {
            let expanded = shellexpand::tilde(dir);
            command.current_dir(expanded.as_ref());
        }

        // Set environment variables
        for (key, value) in &task.env {
            command.env(key, value);
        }

        if !self.verbose {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        let output = command.output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(anyhow::anyhow!(
                "Command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    /// Run a sudo command with keychain support
    async fn run_sudo_command(&self, args: &[String], keychain_label: &str) -> Result<String> {
        // Helper to actually execute the sudo command once authentication timestamp is valid.
        fn run_actual(args: &[String]) -> Result<String> {
            let output = Command::new("sudo")
                .args(args)
                .output()
                .context("Failed to execute sudo command")?;
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err(anyhow::anyhow!(
                    "Command failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ))
            }
        }

        // 1. If sudo timestamp is already cached, just run the command.
        if Command::new("sudo")
            .arg("-n")
            .arg("true")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return run_actual(args);
        }

        // 2. Try keychain password (if stored) to refresh sudo timestamp.
        if let Ok(password) = keychain::get_password(keychain_label) {
            if authenticate_sudo(&password).await? {
                return run_actual(args);
            }
        }

        // 3. Prompt user for password
        let password = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter sudo password")
            .interact()?;

        if !authenticate_sudo(&password).await? {
            return Err(anyhow::anyhow!("Failed to authenticate sudo"));
        }

        // 4. Optionally save password into keychain
        if !keychain::entry_exists(keychain_label) {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Save password to keychain for future use?")
                .default(true)
                .interact()?
            {
                keychain::save_password(keychain_label, &password)?;
            }
        }

        run_actual(args)
    }
}

/// Authenticate sudo with password
async fn authenticate_sudo(password: &str) -> Result<bool> {
    use tokio::io::AsyncWriteExt;
    use tokio::process::Command as TokioCommand;

    let mut child = TokioCommand::new("sudo")
        .arg("-S")
        .arg("true")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(format!("{}\n", password).as_bytes())
            .await?;
    }

    let status = child.wait().await?;
    Ok(status.success())
}

/// Format duration for display
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{}m {}s", secs / 60, secs % 60)
    }
}

fn format_group_label(name: &str, icon: &str) -> String {
    if icon.trim().is_empty() {
        name.to_string()
    } else {
        format!("{} {}", icon, name)
    }
}

fn format_task_label(name: &str, icon: &str) -> String {
    let icon = icon.trim();
    if icon.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", icon, name)
    }
}
