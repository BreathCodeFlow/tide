use anyhow::Result;
use colored::*;
use std::process::Command;

/// Print the Tide banner
pub fn print_banner() {
    let banner = r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║     ████████╗██╗██████╗ ███████╗                          ║
║     ╚══██╔══╝██║██╔══██╗██╔════╝                          ║
║        ██║   ██║██║  ██║█████╗                            ║
║        ██║   ██║██║  ██║██╔══╝                            ║
║        ██║   ██║██████╔╝███████╗                          ║
║        ╚═╝   ╚═╝╚═════╝ ╚══════╝                          ║
║                                                           ║
║        🌊  Refresh your system with the update wave       ║
║                         v1.0.0                            ║
╚═══════════════════════════════════════════════════════════╝"#;

    println!("{}", banner.bright_cyan());
}

/// Display system information
pub fn display_system_info() -> Result<()> {
    println!("\n{}", "📊 System Information".bright_blue().bold());
    println!("{}", "─".repeat(60).dimmed());

    // Disk space
    if let Ok(output) = Command::new("df").args(&["-h", "/"]).output() {
        if output.status.success() {
            let lines = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = lines.lines().nth(1) {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 5 {
                    println!(
                        "  💾 Disk: {} used of {} ({})",
                        parts[2].bright_white(),
                        parts[1].bright_white(),
                        parts[4].bright_yellow()
                    );
                }
            }
        }
    }

    // Battery status
    if let Ok(output) = Command::new("pmset").args(&["-g", "batt"]).output() {
        if output.status.success() {
            let info = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = info.lines().nth(1) {
                if let Some(pct_start) = line.find(char::is_numeric) {
                    if let Some(pct_end) = line[pct_start..].find('%') {
                        let pct = &line[pct_start..pct_start + pct_end];
                        let status = if line.contains("charging") {
                            "charging ⚡".yellow()
                        } else if line.contains("charged") {
                            "charged ✅".green()
                        } else {
                            "battery 🔋".normal()
                        };
                        println!("  🔋 Power: {}% {}", pct.bright_white(), status);
                    }
                }
            }
        }
    }

    // macOS version
    if let Ok(output) = Command::new("sw_vers").arg("-productVersion").output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("  🍎 macOS: {}", version.bright_white());
        }
    }

    // Uptime
    if let Ok(output) = Command::new("uptime").output() {
        if output.status.success() {
            let uptime = String::from_utf8_lossy(&output.stdout);
            if let Some(up_pos) = uptime.find("up ") {
                let up_str = &uptime[up_pos + 3..];
                if let Some(comma_pos) = up_str.find(',') {
                    println!("  ⏱️  Uptime: {}", up_str[..comma_pos].bright_white());
                }
            }
        }
    }

    Ok(())
}

/// Fetch and display weather information
pub async fn get_weather() -> Option<String> {
    let response = reqwest::blocking::get("https://wttr.in?format=%l:+%c+%t+%w+%h")
        .ok()?
        .text()
        .ok()?;

    if !response.is_empty() && !response.contains("Unknown") {
        Some(response.trim().to_string())
    } else {
        None
    }
}

/// Display weather information
pub async fn display_weather() {
    if let Some(weather) = get_weather().await {
        println!("\n{}", "🌤️  Weather".bright_blue().bold());
        println!("{}", "─".repeat(60).dimmed());
        println!("  {}", weather.bright_white());
    }
}
