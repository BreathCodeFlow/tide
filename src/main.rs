mod cli;
mod config;
mod error;
mod executor;
mod keychain;
mod logger;
mod notifications;
mod ui;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use dialoguer::{Confirm, theme::ColorfulTheme};
use futures::future::join_all;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

use cli::Args;
use config::{Config, Settings};
use executor::{TaskExecutor, TaskResult, TaskStatus};
use logger::Logger;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    if args.init {
        return init_config(args.config.as_ref());
    }

    if std::env::consts::OS != "macos" {
        eprintln!("{}", "❌ This tool is for macOS only!".red().bold());
        std::process::exit(1);
    }

    let config_path = Config::resolve_path(args.config.as_ref())?;
    let config = Config::load(Some(&config_path))?;

    if args.list {
        list_tasks(&config, &args);
        display_config_path(&config_path)?;
        return Ok(());
    }

    setup_environment();

    let logger = match init_logger(&config.settings, &config_path)? {
        Some((logger, path)) => {
            if !args.quiet {
                println!(
                    "{}",
                    format!("📝 Task output will be logged to {}", path.display()).dimmed()
                );
            }
            Some(logger)
        }
        None => None,
    };

    let weather_task = if !args.quiet && config.settings.show_weather {
        Some(tokio::spawn(ui::fetch_weather()))
    } else {
        None
    };

    if !args.quiet && config.settings.show_banner {
        ui::print_banner();
    }

    let mut all_tasks = Vec::new();
    for group in &config.groups {
        if !group.enabled {
            continue;
        }

        if let Some(ref groups) = args.groups
            && !groups.contains(&group.name)
        {
            continue;
        }
        if let Some(ref skip) = args.skip_groups
            && skip.contains(&group.name)
        {
            continue;
        }

        for task in &group.tasks {
            if task.enabled {
                all_tasks.push((
                    task.clone(),
                    group.name.clone(),
                    group.icon.clone(),
                    group.parallel,
                ));
            }
        }
    }

    if all_tasks.is_empty() {
        println!("{}", "No tasks to run!".yellow());
        return Ok(());
    }

    if !args.force && !args.quiet {
        println!(
            "\n{}",
            format!("📦 Ready to run {} tasks", all_tasks.len()).bright_blue()
        );

        if args.dry_run {
            println!("{}", "🔸 DRY RUN MODE - No changes will be made".yellow());
        }

        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Continue?")
            .default(true)
            .interact()?
        {
            println!("{}", "Cancelled by user".yellow());
            return Ok(());
        }
    }

    let show_progress = config.settings.show_progress && !args.quiet;
    let executor = Arc::new(TaskExecutor::new(
        args.dry_run,
        args.verbose || config.settings.verbose,
        config.settings.desktop_notifications && !args.quiet,
        show_progress,
        logger.clone(),
    ));
    let start_time = Instant::now();
    let mut results = Vec::new();

    let keychain_label = config
        .settings
        .keychain_label
        .as_deref()
        .unwrap_or("tide-sudo");

    // Pre-authenticate sudo to prevent tasks from hanging
    // This helps even if tasks don't have sudo: true but internally call sudo
    // We do this proactively unless in dry-run mode
    if !args.dry_run && !args.quiet {
        // Only attempt if sudo is available and we're not running quietly
        if keychain::command_exists("sudo") {
            match executor.ensure_sudo_auth(keychain_label).await {
                Ok(_) => {
                    // Successfully authenticated or timestamp was valid
                }
                Err(e) => {
                    // Sudo auth failed - warn but don't exit
                    // Some tasks might not need sudo
                    eprintln!(
                        "{}",
                        format!("⚠️  Sudo authentication failed: {}", e).yellow()
                    );
                    eprintln!(
                        "{}",
                        "   Tasks requiring sudo may fail or timeout.".yellow()
                    );
                }
            }
        }
    }

    let mut sequential_tasks = Vec::new();
    let mut parallel_tasks = Vec::new();

    for (task, group, group_icon, is_parallel) in all_tasks {
        if is_parallel || (config.settings.parallel_execution && !task.sudo) {
            parallel_tasks.push((task, group, group_icon));
        } else {
            sequential_tasks.push((task, group, group_icon));
        }
    }

    for (task, group, group_icon) in sequential_tasks {
        let pb = executor.new_spinner();
        let result = executor
            .execute_task(task, group, group_icon, pb, keychain_label)
            .await;

        if result.status == TaskStatus::Failed && config.settings.skip_optional_on_error {
            println!(
                "{}",
                "⚠️  Skipping remaining optional tasks due to failure".yellow()
            );
            break;
        }

        results.push(result);
    }

    if !parallel_tasks.is_empty() {
        let semaphore = Arc::new(Semaphore::new(
            args.parallel.min(config.settings.parallel_limit),
        ));
        let mut handles = Vec::new();

        for (task, group, group_icon) in parallel_tasks {
            let executor_clone = Arc::clone(&executor);
            let semaphore_clone = Arc::clone(&semaphore);
            let keychain_label = keychain_label.to_string();
            let group_clone = group.clone();
            let icon_clone = group_icon.clone();

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                let pb = executor_clone.new_spinner();
                executor_clone
                    .execute_task(task, group_clone, icon_clone, pb, &keychain_label)
                    .await
            });

            handles.push(handle);
        }

        let parallel_results = join_all(handles).await;
        for task_result in parallel_results.into_iter().flatten() {
            results.push(task_result);
        }
    }

    let total_duration = start_time.elapsed();
    display_results(&results, total_duration);

    // Send completion notification if all tasks succeeded
    let success_count = results
        .iter()
        .filter(|r| r.status == TaskStatus::Success)
        .count();
    let failed_count = results
        .iter()
        .filter(|r| r.status == TaskStatus::Failed)
        .count();

    if failed_count == 0 && success_count > 0 {
        let _ = executor
            .notifier
            .notify_all_tasks_complete(success_count, total_duration.as_secs());
    }

    if !args.quiet && config.settings.show_system_info {
        ui::display_system_info()?;
    }

    if let Some(handle) = weather_task {
        let status = match handle.await {
            Ok(status) => status,
            Err(err) => ui::WeatherStatus::Error(format!("Runtime error: {err}")),
        };
        ui::render_weather(status);
    }

    Ok(())
}

fn display_config_path(path: &Path) -> Result<()> {
    println!(
        "{} {}",
        "Using config file:".bright_blue().bold(),
        path.display()
    );
    Ok(())
}

fn init_logger(settings: &Settings, config_path: &Path) -> Result<Option<(Arc<Logger>, PathBuf)>> {
    let raw_path = match settings.log_file_path() {
        Some(path) => path,
        None => return Ok(None),
    };

    let expanded = shellexpand::tilde(raw_path);
    let mut resolved = PathBuf::from(expanded.as_ref());

    if resolved.is_relative() && let Some(parent) = config_path.parent() {
        resolved = parent.join(resolved);
    }

    let logger = Arc::new(Logger::new(&resolved)?);
    Ok(Some((logger, resolved)))
}

fn init_config(path: Option<&PathBuf>) -> Result<()> {
    let config_dir = if let Some(p) = path {
        p.parent().unwrap().to_path_buf()
    } else {
        dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("tide")
    };

    fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("config.toml");

    if config_path.exists()
        && !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Config file already exists. Overwrite?")
            .default(false)
            .interact()?
    {
        return Ok(());
    }

    let default_config = Config::default();
    let toml_str = toml::to_string_pretty(&default_config)?;
    fs::write(&config_path, toml_str)?;

    println!(
        "{}",
        format!("✓ Config created: {}", config_path.display()).green()
    );
    println!("Edit it with: nano {}", config_path.display());

    Ok(())
}

fn list_tasks(config: &Config, args: &Args) {
    println!("{}", "📋 Configured Tasks".bright_blue().bold());
    println!("{}", "═".repeat(60).bright_blue());

    for group in &config.groups {
        if let Some(ref groups) = args.groups
            && !groups.contains(&group.name)
        {
            continue;
        }
        if let Some(ref skip) = args.skip_groups
            && skip.contains(&group.name)
        {
            continue;
        }

        let enabled_icon = if group.enabled {
            "✓".green()
        } else {
            "✗".red()
        };
        println!(
            "\n{} {} {}",
            group.icon,
            group.name.bright_white().bold(),
            enabled_icon
        );
        if !group.description.is_empty() {
            println!("  {}", group.description.dimmed());
        }

        for task in &group.tasks {
            let enabled_icon = if task.enabled {
                "✓".green()
            } else {
                "✗".red()
            };
            let required_icon = if task.required { "🔴" } else { "⚪" };
            let sudo_icon = if task.sudo { "🔐" } else { "  " };

            print!(
                "  {} {} {} {} {}",
                enabled_icon,
                required_icon,
                sudo_icon,
                task.icon,
                task.name.bright_white()
            );

            if args.verbose && !task.description.is_empty() {
                println!();
                println!("      {}", task.description.dimmed());
            } else {
                println!();
            }

            if args.verbose {
                println!("      Command: {}", task.command.join(" ").dimmed());
            }
        }
    }

    println!("\n{}", "Legend:".dimmed());
    println!("  {} Enabled/Disabled", "✓/✗".dimmed());
    println!("  {} Required task", "🔴".dimmed());
    println!("  {} Optional task", "⚪".dimmed());
    println!("  {} Requires sudo", "🔐".dimmed());
    println!();
}

fn display_results(results: &[TaskResult], total_duration: Duration) {
    let success = results
        .iter()
        .filter(|r| r.status == TaskStatus::Success)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == TaskStatus::Failed)
        .count();
    let skipped = results
        .iter()
        .filter(|r| r.status == TaskStatus::Skipped)
        .count();

    println!("\n{}", "📊 Summary".bright_blue().bold());
    println!("{}", "─".repeat(60).dimmed());

    println!(
        "  {} Success  {} Failed  {} Skipped  ⏱️  Total: {}",
        format!("✓ {}", success).green(),
        format!("✗ {}", failed).red(),
        format!("○ {}", skipped).yellow(),
        format_duration(total_duration).bright_white()
    );

    if let Some(longest_task) = results.iter().max_by_key(|r| r.duration) {
        let group_label = format_group_display(&longest_task.group, &longest_task.group_icon);
        println!(
            "  Longest task: {} [{} in {}]",
            format_duration(longest_task.duration).bright_white(),
            longest_task.name.bright_white(),
            group_label.dimmed()
        );
    }

    if failed > 0 {
        println!("\n{}", "Failed tasks:".red().bold());
        for result in results.iter().filter(|r| r.status == TaskStatus::Failed) {
            let group_label = format_group_display(&result.group, &result.group_icon);
            println!("  ✗ {} - {}", result.name.red(), group_label.dimmed());
            if let Some(output) = &result.output
                && !output.is_empty()
            {
                println!("    {}", output.dimmed());
            }
        }
    }
}

fn setup_environment() {
    if Path::new("/opt/homebrew/bin/brew").exists() {
        prepend_to_path("/opt/homebrew/bin");
    } else if Path::new("/usr/local/bin/brew").exists() {
        prepend_to_path("/usr/local/bin");
    }

    if let Some(home) = dirs::home_dir() {
        let local_bin = home.join(".local/bin");
        if local_bin.exists() {
            prepend_to_path(local_bin);
        }
    }
}

fn prepend_to_path<P: AsRef<Path>>(dir: P) {
    let dir = dir.as_ref();
    if !dir.exists() {
        return;
    }

    let mut new_path = OsString::from(dir.as_os_str());
    if let Some(current) = env::var_os("PATH") && !current.is_empty() {
        new_path.push(":");
        new_path.push(current);
    }
    unsafe {
        env::set_var("PATH", new_path);
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else {
        format!("{}m {}s", secs / 60, secs % 60)
    }
}

fn format_group_display(name: &str, icon: &str) -> String {
    let icon = icon.trim();
    if icon.is_empty() {
        name.to_string()
    } else {
        format!("{} {}", icon, name)
    }
}
