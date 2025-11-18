use anyhow::{Context, Result};
use colored::Colorize;
use dialoguer::{Confirm, Password, theme::ColorfulTheme};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::TaskConfig;
use crate::keychain;
use crate::logger::Logger;
use crate::notifications::NotificationManager;

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
    pub multi_progress: Option<Arc<MultiProgress>>,
    pub dry_run: bool,
    pub verbose: bool,
    pub notifier: Arc<NotificationManager>,
    logger: Option<Arc<Logger>>,
    show_progress: bool,
}

impl TaskExecutor {
    /// Create a new task executor
    pub fn new(
        dry_run: bool,
        verbose: bool,
        notifications_enabled: bool,
        show_progress: bool,
        logger: Option<Arc<Logger>>,
    ) -> Self {
        Self {
            multi_progress: show_progress.then(|| Arc::new(MultiProgress::new())),
            dry_run,
            verbose,
            notifier: Arc::new(NotificationManager::new(notifications_enabled)),
            logger,
            show_progress,
        }
    }

    fn update_progress(&self, pb: &ProgressBar, message: &str) {
        if self.show_progress {
            pb.set_message(message.to_string());
        } else {
            println!("{}", message);
        }
    }

    fn finish_progress(&self, pb: &ProgressBar, message: &str) {
        if self.show_progress {
            pb.finish_with_message(message.to_string());
        } else {
            println!("{}", message);
        }
    }

    fn log_line(&self, message: String) {
        if let Some(logger) = &self.logger {
            if let Err(err) = logger.log_line(&message) {
                if self.verbose {
                    eprintln!("{}", format!("Failed to write log entry: {}", err).yellow());
                }
            }
        }
    }

    fn log_task_completion(
        &self,
        group_label: &str,
        task_label: &str,
        status: TaskStatus,
        duration: Duration,
        output: Option<&str>,
    ) {
        if self.logger.is_none() {
            return;
        }

        let status_prefix = match status {
            TaskStatus::Success => "✓ SUCCESS",
            TaskStatus::Failed => "✗ FAILED",
            TaskStatus::Skipped => "○ SKIPPED",
        };
        self.log_line(format!(
            "{} [{}] {} ({})",
            status_prefix,
            group_label,
            task_label,
            format_duration(duration)
        ));

        if let Some(output) = output {
            let trimmed = output.trim();
            if trimmed.is_empty() {
                return;
            }
            if let Some(logger) = &self.logger {
                let header = format!("└ output [{}] {}", group_label, task_label);
                if let Err(err) = logger.log_block(&header, trimmed) {
                    if self.verbose {
                        eprintln!("{}", format!("Failed to write log entry: {}", err).yellow());
                    }
                }
            }
        }
    }

    /// Ensure sudo authentication is valid before executing tasks
    /// This prevents tasks from hanging on password prompts
    /// Returns Ok if auth succeeded or was already valid
    /// Returns Err only if user provided wrong password
    pub async fn ensure_sudo_auth(&self, keychain_label: &str) -> Result<()> {
        // Check if sudo timestamp is already cached
        if Command::new("sudo")
            .arg("-n")
            .arg("true")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            if self.verbose {
                println!("{}", "✓ Sudo timestamp already valid".green());
            }
            return Ok(());
        }

        // Try keychain password to refresh sudo timestamp
        if let Ok(password) = keychain::get_password(keychain_label) {
            if authenticate_sudo(&password).await? {
                if self.verbose {
                    println!("{}", "✓ Sudo authenticated via keychain".green());
                }
                return Ok(());
            } else {
                // Keychain password is wrong/outdated - we'll prompt
                if self.verbose {
                    println!(
                        "{}",
                        "⚠️  Keychain password is outdated, prompting for new password".yellow()
                    );
                }
            }
        }

        // Prompt user for password
        println!(
            "{}",
            "🔐 Some tasks may require sudo privileges.".bright_blue()
        );

        // Send desktop notification
        let _ = self.notifier.notify_sudo_required();

        let password = match Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter sudo password (or press Ctrl+C to skip)")
            .allow_empty_password(true)
            .interact()
        {
            Ok(pwd) if pwd.is_empty() => {
                println!("{}", "Skipping sudo authentication.".yellow());
                return Err(anyhow::anyhow!("User skipped sudo authentication"));
            }
            Ok(pwd) => pwd,
            Err(_) => {
                println!("{}", "Sudo authentication cancelled.".yellow());
                return Err(anyhow::anyhow!("User cancelled sudo authentication"));
            }
        };

        if !authenticate_sudo(&password).await? {
            return Err(anyhow::anyhow!("Invalid sudo password"));
        }

        if self.verbose {
            println!("{}", "✓ Sudo authenticated successfully".green());
        }

        // Optionally save password into keychain
        if !keychain::entry_exists(keychain_label)
            && Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Save password to keychain for future use?")
                .default(true)
                .interact()?
        {
            keychain::save_password(keychain_label, &password)?;
            println!(
                "{}",
                "✓ Password saved to keychain (service: tide-sudo)".green()
            );
        }

        Ok(())
    }

    /// Create a configured spinner progress bar
    pub fn new_spinner(&self) -> ProgressBar {
        if let Some(multi) = &self.multi_progress {
            let pb = multi.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::with_template("{spinner:.cyan} {msg}")
                    .unwrap()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
            );
            pb.enable_steady_tick(Duration::from_millis(120));
            pb
        } else {
            ProgressBar::hidden()
        }
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
        let running_message = format!("{} {}", progress_label.bold(), "Running…".bright_white());
        self.update_progress(&pb, &running_message);

        let mut cmd = task.command.clone();
        if task.sudo && !cmd.is_empty() && cmd[0] != "sudo" {
            cmd.insert(0, "sudo".to_string());
        }
        let command_display = if cmd.is_empty() {
            "<empty command>".to_string()
        } else {
            cmd.join(" ")
        };
        self.log_line(format!(
            "▶ [{}] {} :: {}",
            group_label, task_label, command_display
        ));

        if self.dry_run {
            tokio::time::sleep(Duration::from_millis(100)).await;
            let dry_run_msg = format!(
                "{} {} {}",
                progress_label.bold(),
                "○".yellow(),
                "[dry run]".dimmed()
            );
            self.finish_progress(&pb, &dry_run_msg);
            let duration = start.elapsed();
            let reason = "Dry run - command not executed".to_string();
            self.log_task_completion(
                &group_label,
                &task_label,
                TaskStatus::Skipped,
                duration,
                Some(reason.as_str()),
            );
            return TaskResult {
                name: task_name.clone(),
                group: group_name,
                group_icon,
                status: TaskStatus::Skipped,
                duration,
                output: Some(reason),
            };
        }

        // Check preconditions
        if let Some(check_cmd) = &task.check_command
            && !keychain::command_exists(check_cmd)
        {
            let skip_msg = format!(
                "{} {}",
                progress_label.bold(),
                "[skipped: command not found]".dimmed()
            );
            self.finish_progress(&pb, &skip_msg);
            let duration = start.elapsed();
            let reason = format!("Command '{}' not found", check_cmd);
            self.log_task_completion(
                &group_label,
                &task_label,
                TaskStatus::Skipped,
                duration,
                Some(reason.as_str()),
            );
            return TaskResult {
                name: task_name.clone(),
                group: group_name,
                group_icon,
                status: TaskStatus::Skipped,
                duration,
                output: Some(reason),
            };
        }

        if let Some(check_path) = &task.check_path {
            let expanded = shellexpand::tilde(check_path);
            if !Path::new(expanded.as_ref()).exists() {
                let skip_msg = format!(
                    "{} {}",
                    progress_label.bold(),
                    "[skipped: path not found]".dimmed()
                );
                self.finish_progress(&pb, &skip_msg);
                let duration = start.elapsed();
                let reason = format!("Path '{}' not found", check_path);
                self.log_task_completion(
                    &group_label,
                    &task_label,
                    TaskStatus::Skipped,
                    duration,
                    Some(reason.as_str()),
                );
                return TaskResult {
                    name: task_name.clone(),
                    group: group_name,
                    group_icon,
                    status: TaskStatus::Skipped,
                    duration,
                    output: Some(reason),
                };
            }
        }

        // Warn if command might internally call sudo (heuristic check)
        if !task.sudo && self.verbose && !cmd.is_empty() {
            let cmd_str = cmd.join(" ").to_lowercase();
            if cmd_str.contains("sudo") {
                pb.println(format!(
                    "{}",
                    format!(
                        "⚠️  Task '{}' may call sudo internally. Consider setting 'sudo: true'",
                        task_name
                    )
                    .yellow()
                ));
            }
        }

        // Execute command
        let result = if cmd.first().map(|s| s.as_str()) == Some("sudo") {
            self.run_sudo_command(&cmd[1..], keychain_label).await
        } else {
            self.run_command(&cmd, &task, &task_name, &group_name).await
        };

        let (status, output) = match result {
            Ok(output) => (TaskStatus::Success, Some(output)),
            Err(e) if task.required => {
                // Send notification for failed required task
                let _ = self
                    .notifier
                    .notify_task_failed(&task_name, &group_name, &e.to_string());
                (TaskStatus::Failed, Some(e.to_string()))
            }
            Err(e) => (TaskStatus::Skipped, Some(e.to_string())),
        };

        let duration = start.elapsed();
        let status_icon = match status {
            TaskStatus::Success => "✓".green(),
            TaskStatus::Failed => "✗".red(),
            TaskStatus::Skipped => "○".yellow(),
        };

        let completion_message = format!(
            "{} {} {}",
            progress_label.bold(),
            status_icon,
            format!("({})", format_duration(duration)).dimmed()
        );
        self.finish_progress(&pb, &completion_message);
        self.log_task_completion(
            &group_label,
            &task_label,
            status,
            duration,
            output.as_deref(),
        );

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
    async fn run_command(
        &self,
        cmd: &[String],
        task: &TaskConfig,
        task_name: &str,
        group_name: &str,
    ) -> Result<String> {
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

        // CRITICAL: Redirect stdin to /dev/null to prevent blocking on password prompts
        // This prevents commands from hanging if they internally require interactive input
        command.stdin(Stdio::null());

        if !self.verbose {
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
        }

        // Apply timeout if specified in task config
        let command_future = tokio::task::spawn_blocking(move || command.output());
        let timeout_secs = task.timeout.unwrap_or(300);

        let output = match tokio::time::timeout(Duration::from_secs(timeout_secs), command_future)
            .await
        {
            Ok(Ok(result)) => result?,
            Ok(Err(e)) => return Err(anyhow::anyhow!("Command execution error: {}", e)),
            Err(_) => {
                // Send notification that task timed out (likely waiting for input)
                let _ = self
                    .notifier
                    .notify_interactive_input_detected(task_name, group_name);
                let _ = self
                    .notifier
                    .notify_task_timeout(task_name, group_name, timeout_secs);

                return Err(anyhow::anyhow!(
                    "Command timed out after {} seconds. This may indicate the command is waiting for input (like sudo password). Consider setting 'sudo: true' or 'timeout: <seconds>' in the task config.",
                    timeout_secs
                ));
            }
        };

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
        if let Ok(password) = keychain::get_password(keychain_label)
            && authenticate_sudo(&password).await?
        {
            return run_actual(args);
        }

        // 3. Prompt user for password
        let password = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter sudo password")
            .interact()?;

        if !authenticate_sudo(&password).await? {
            return Err(anyhow::anyhow!("Failed to authenticate sudo"));
        }

        // 4. Optionally save password into keychain
        if !keychain::entry_exists(keychain_label)
            && Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Save password to keychain for future use?")
                .default(true)
                .interact()?
        {
            keychain::save_password(keychain_label, &password)?;
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
