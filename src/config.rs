use crate::error::TideError;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Main configuration structure
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub settings: Settings,
    #[serde(default)]
    pub groups: Vec<TaskGroup>,
}

/// Global settings
#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default = "default_true")]
    pub show_banner: bool,
    #[serde(default = "default_true")]
    pub show_weather: bool,
    #[serde(default = "default_true")]
    pub show_system_info: bool,
    #[serde(default = "default_false")]
    pub show_progress: bool,
    #[serde(default = "default_false")]
    pub parallel_execution: bool,
    #[serde(default = "default_parallel_limit")]
    pub parallel_limit: usize,
    #[serde(default = "default_false")]
    pub skip_optional_on_error: bool,
    #[serde(default)]
    pub keychain_label: Option<String>,
    #[serde(default = "default_true")]
    pub use_colors: bool,
    #[serde(default = "default_false")]
    pub verbose: bool,
    #[serde(default)]
    pub log_file: Option<String>,
    #[serde(default = "default_true")]
    pub desktop_notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            show_banner: true,
            show_weather: true,
            show_system_info: true,
            show_progress: true,
            parallel_execution: false,
            parallel_limit: 4,
            skip_optional_on_error: false,
            keychain_label: Some("tide-sudo".to_string()),
            use_colors: true,
            verbose: false,
            log_file: None,
            desktop_notifications: true,
        }
    }
}

impl Settings {
    /// Return the configured log file path, ignoring empty values.
    pub fn log_file_path(&self) -> Option<&str> {
        self.log_file
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
    }
}

/// Task group configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskGroup {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub parallel: bool,
    #[serde(default)]
    pub tasks: Vec<TaskConfig>,
}

/// Individual task configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TaskConfig {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    pub command: Vec<String>,
    #[serde(default = "default_true")]
    pub required: bool,
    #[serde(default = "default_false")]
    pub sudo: bool,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub check_command: Option<String>,
    #[serde(default)]
    pub check_path: Option<String>,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub timeout: Option<u64>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub working_dir: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_parallel_limit() -> usize {
    4
}

impl Config {
    /// Resolve the path that should be used for the configuration file
    pub fn resolve_path(path: Option<&PathBuf>) -> Result<PathBuf> {
        if let Some(p) = path {
            Ok(p.clone())
        } else {
            Self::default_config_path()
        }
    }

    /// Load configuration from file or use default path
    pub fn load(path: Option<&PathBuf>) -> Result<Self> {
        let config_path = Self::resolve_path(path)?;

        if !config_path.exists() {
            return Err(TideError::Config(format!(
                "Config file not found: {}\nRun 'tide --init' to create one.",
                config_path.display()
            ))
            .into());
        }

        let contents = fs::read_to_string(&config_path).context(format!(
            "Failed to read config file: {}",
            config_path.display()
        ))?;

        toml::from_str(&contents).context("Failed to parse config file")
    }

    /// Get default configuration path
    pub fn default_config_path() -> Result<PathBuf> {
        Ok(dirs::config_dir()
            .context("Could not determine config directory")?
            .join("tide")
            .join("config.toml"))
    }

    /// Create default configuration
    pub fn default() -> Self {
        Self {
            settings: Settings::default(),
            groups: vec![
                TaskGroup {
                    name: "System Updates".to_string(),
                    icon: "🍎".to_string(),
                    enabled: true,
                    description: "macOS system updates".to_string(),
                    parallel: false,
                    tasks: vec![TaskConfig {
                        name: "macOS Updates".to_string(),
                        icon: "🍎".to_string(),
                        command: vec![
                            "softwareupdate".to_string(),
                            "--install".to_string(),
                            "--all".to_string(),
                        ],
                        required: true,
                        sudo: true,
                        enabled: true,
                        check_command: Some("softwareupdate".to_string()),
                        check_path: None,
                        description: "Install macOS system updates".to_string(),
                        timeout: Some(3600),
                        env: HashMap::new(),
                        working_dir: None,
                    }],
                },
                TaskGroup {
                    name: "Homebrew".to_string(),
                    icon: "🍺".to_string(),
                    enabled: true,
                    description: "Homebrew package manager".to_string(),
                    parallel: false,
                    tasks: vec![
                        TaskConfig {
                            name: "Update Formulae".to_string(),
                            icon: "📦".to_string(),
                            command: vec!["brew".to_string(), "update".to_string()],
                            required: true,
                            sudo: false,
                            enabled: true,
                            check_command: Some("brew".to_string()),
                            check_path: None,
                            description: "Update Homebrew package definitions".to_string(),
                            timeout: Some(300),
                            env: HashMap::new(),
                            working_dir: None,
                        },
                        TaskConfig {
                            name: "Upgrade Packages".to_string(),
                            icon: "⬆️".to_string(),
                            command: vec!["brew".to_string(), "upgrade".to_string()],
                            required: true,
                            sudo: false,
                            enabled: true,
                            check_command: Some("brew".to_string()),
                            check_path: None,
                            description: "Upgrade all outdated packages".to_string(),
                            timeout: Some(1200),
                            env: HashMap::new(),
                            working_dir: None,
                        },
                    ],
                },
            ],
        }
    }
}
