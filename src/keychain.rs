use anyhow::Result;
use std::process::{Command, Stdio};

/// Check if a keychain entry exists
pub fn entry_exists(label: &str) -> bool {
    Command::new("security")
        .args(&["find-generic-password", "-s", label, "-a", "root"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get password from keychain
pub fn get_password(label: &str) -> Result<String> {
    let output = Command::new("security")
        .args(&["find-generic-password", "-s", label, "-a", "root", "-w"])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!("Keychain entry not found"))
    }
}

/// Save password to keychain
pub fn save_password(label: &str, password: &str) -> Result<()> {
    let status = Command::new("security")
        .args(&[
            "add-generic-password",
            "-s",
            label,
            "-a",
            "root",
            "-w",
            password,
        ])
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Failed to save password to keychain"))
    }
}

/// Check command existence in PATH
pub fn command_exists(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}
