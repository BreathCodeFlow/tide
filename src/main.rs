mod cli;
mod config;
mod error;
mod executor;
mod keychain;
mod ui;

use anyhow::Result;
use clap::Parser;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm};
use dirs;
use futures::future::join_all;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

use cli::Args;
use config::Config;
use executor::{TaskExecutor, TaskResult, TaskStatus};

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

    let config = Config::load(args.config.as_ref())?;

    if args.list {
        list_tasks(&config, &args);
        return Ok(());
    }

    setup_environment();

    if !args.quiet && config.settings.show_banner {
        ui::print_banner();
    }

    let mut all_tasks = Vec::new();
    for group in &config.groups {
        if !group.enabled {
            continue;
        }

        if let Some(ref groups) = args.groups {
            if !groups.contains(&group.name) {
                continue;
            }
        }
        if let Some(ref skip) = args.skip_groups {
            if skip.contains(&group.name) {
                continue;
            }
        }

        for task in &group.tasks {
            if task.enabled {
                all_tasks.push((task.clone(), group.name.clone(), group.parallel));
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

    let executor = TaskExecutor::new(args.dry_run, args.verbose || config.settings.verbose);
    let start_time = Instant::now();
    let mut results = Vec::new();

    let keychain_label = config
        .settings
        .keychain_label
        .as_deref()
        .unwrap_or("tide-sudo");

    let mut sequential_tasks = Vec::new();
    let mut parallel_tasks = Vec::new();

    for (task, group, is_parallel) in all_tasks {
        if is_parallel || (config.settings.parallel_execution && !task.sudo) {
            parallel_tasks.push((task, group));
        } else {
            sequential_tasks.push((task, group));
        }
    }

    for (task, group) in sequential_tasks {
        let pb = executor.multi_progress.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );

        let result = executor.execute_task(task, group, pb, keychain_label).await;

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
        let executor_arc = Arc::new(executor);
        let mut handles = Vec::new();

        for (task, group) in parallel_tasks {
            let pb = executor_arc.multi_progress.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.cyan} {msg}")
                    .unwrap(),
            );

            let executor_clone = Arc::clone(&executor_arc);
            let semaphore_clone = Arc::clone(&semaphore);
            let keychain_label = keychain_label.to_string();

            let handle = tokio::spawn(async move {
                let _permit = semaphore_clone.acquire().await.unwrap();
                executor_clone
                    .execute_task(task, group, pb, &keychain_label)
                    .await
            });

            handles.push(handle);
        }

        let parallel_results = join_all(handles).await;
        for result in parallel_results {
            if let Ok(task_result) = result {
                results.push(task_result);
            }
        }
    }

    let total_duration = start_time.elapsed();
    display_results(&results, total_duration);

    if !args.quiet && config.settings.show_system_info {
        ui::display_system_info()?;
    }

    if !args.quiet && config.settings.show_weather {
        ui::display_weather().await;
    }

    Ok(())
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

    if config_path.exists() {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Config file already exists. Overwrite?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
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
        if let Some(ref groups) = args.groups {
            if !groups.contains(&group.name) {
                continue;
            }
        }
        if let Some(ref skip) = args.skip_groups {
            if skip.contains(&group.name) {
                continue;
            }
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

    if failed > 0 {
        println!("\n{}", "Failed tasks:".red().bold());
        for result in results.iter().filter(|r| r.status == TaskStatus::Failed) {
            println!("  ✗ {} - {}", result.name.red(), result.group.dimmed());
            if let Some(output) = &result.output {
                if !output.is_empty() {
                    println!("    {}", output.dimmed());
                }
            }
        }
    }
}

fn setup_environment() {
    if Path::new("/opt/homebrew/bin/brew").exists() {
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("/opt/homebrew/bin:{}", path));
    } else if Path::new("/usr/local/bin/brew").exists() {
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("/usr/local/bin:{}", path));
    }

    if let Some(home) = dirs::home_dir() {
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{}/.local/bin:{}", home.display(), path));
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
