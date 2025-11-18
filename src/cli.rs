use clap::Parser;
use std::path::PathBuf;

/// CLI Arguments for Tide
#[derive(Parser, Debug)]
#[command(name = "tide")]
#[command(about = "🌊 Tide - Refresh your system with the update wave", long_about = None)]
#[command(version)]
pub struct Args {
    /// Run in quiet mode (no banner, minimal output)
    #[arg(short, long)]
    pub quiet: bool,

    /// Run in dry-run mode (show what would be executed)
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Run specific groups only (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    pub groups: Option<Vec<String>>,

    /// Skip specific groups (comma-separated)
    #[arg(short = 'x', long, value_delimiter = ',')]
    pub skip_groups: Option<Vec<String>>,

    /// Maximum parallel tasks (default: 4)
    #[arg(short = 'j', long, default_value = "4")]
    pub parallel: usize,

    /// Config file path (default: ~/.config/tide/config.toml)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Generate default config and exit
    #[arg(long)]
    pub init: bool,

    /// List all configured tasks and exit
    #[arg(short, long)]
    pub list: bool,

    /// Force run without confirmations
    #[arg(short, long)]
    pub force: bool,

    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
