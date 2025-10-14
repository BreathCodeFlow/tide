use anyhow::Result;
use colored::Colorize;
use std::process::Command;
use std::time::Duration;

const DIVIDER_WIDTH: usize = 60;

/// Print the Tide banner
pub fn print_banner() {
    let version = env!("CARGO_PKG_VERSION");
    let banner = format!(
        r#"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║               ████████╗██╗██████╗ ███████╗                ║
║               ╚══██╔══╝██║██╔══██╗██╔════╝                ║
║                  ██║   ██║██║  ██║█████╗                  ║
║                  ██║   ██║██║  ██║██╔══╝                  ║
║                  ██║   ██║██████╔╝███████╗                ║
║                  ╚═╝   ╚═╝╚═════╝ ╚══════╝                ║
║                                                           ║
║        🌊  Refresh your system with the update wave       ║
║                        v{:>8}                           ║
╚═══════════════════════════════════════════════════════════╝"#,
        version
    );

    println!("{}", banner.bright_cyan());
}

/// Display system information
pub fn display_system_info() -> Result<()> {
    println!("\n{}", "📊 System Information".bright_blue().bold());
    println!("{}", "─".repeat(DIVIDER_WIDTH).dimmed());

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

/// Result of a weather lookup
#[derive(Debug)]
pub enum WeatherStatus {
    Available(String),
    NoData(&'static str),
    Error(String),
}

/// Fetch weather information with a short timeout
pub async fn fetch_weather() -> WeatherStatus {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent(format!("tide-cli/{}", env!("CARGO_PKG_VERSION")))
        .build()
    {
        Ok(client) => client,
        Err(err) => return WeatherStatus::Error(format!("HTTP client failed: {err}")),
    };

    let response = match client
        .get("https://wttr.in")
        .query(&[("format", "%l: %c %t %w %h")])
        .header("Accept", "text/plain")
        .header("Cache-Control", "no-cache")
        .send()
        .await
    {
        Ok(response) => response,
        Err(err) => return WeatherStatus::Error(format!("Request error: {err}")),
    };

    if !response.status().is_success() {
        return WeatherStatus::Error(format!("Service returned status {}", response.status()));
    }

    let body = match response.text().await {
        Ok(text) => text,
        Err(err) => return WeatherStatus::Error(format!("Response decode error: {err}")),
    };

    let trimmed = body.trim();
    if trimmed.is_empty() || trimmed.contains("Unknown") {
        WeatherStatus::NoData("Weather data currently unavailable.")
    } else {
        WeatherStatus::Available(trimmed.to_string())
    }
}

/// Display weather information
pub fn render_weather(status: WeatherStatus) {
    println!("\n{}", "🌤️  Weather".bright_blue().bold());
    println!("{}", "─".repeat(DIVIDER_WIDTH).dimmed());

    match status {
        WeatherStatus::Available(summary) => println!("  {}", summary.bright_white()),
        WeatherStatus::NoData(message) => println!("  {}", message.dimmed()),
        WeatherStatus::Error(reason) => println!(
            "  {}",
            format!("Unable to fetch weather data ({reason}).").dimmed()
        ),
    }
}
