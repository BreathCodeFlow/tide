use anyhow::Result;
use notify_rust::{Notification, Timeout};

/// Notification manager for desktop alerts
pub struct NotificationManager {
    enabled: bool,
}

impl NotificationManager {
    /// Create a new notification manager
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Send a notification that a task is waiting for interactive input
    pub fn notify_interactive_input_detected(
        &self,
        task_name: &str,
        group_name: &str,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        Notification::new()
            .summary("🌊 Tide - Interaction Required")
            .body(&format!(
                "Task '{}' (group: {}) appears to be waiting for interactive input.\n\
                 Check your terminal or consider setting 'sudo: true' in config.",
                task_name, group_name
            ))
            .icon("dialog-warning")
            .timeout(Timeout::Milliseconds(10000)) // 10 seconds
            .show()?;

        Ok(())
    }

    /// Send a notification that a task timed out
    pub fn notify_task_timeout(
        &self,
        task_name: &str,
        group_name: &str,
        timeout: u64,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        Notification::new()
            .summary("⚠️ Tide - Task Timeout")
            .body(&format!(
                "Task '{}' (group: {}) timed out after {} seconds.\n\
                 It may be waiting for input or stuck.",
                task_name, group_name, timeout
            ))
            .icon("dialog-error")
            .timeout(Timeout::Milliseconds(8000)) // 8 seconds
            .show()?;

        Ok(())
    }

    /// Send a notification that a task failed
    pub fn notify_task_failed(&self, task_name: &str, group_name: &str, error: &str) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let error_preview = if error.len() > 100 {
            format!("{}...", &error[..100])
        } else {
            error.to_string()
        };

        Notification::new()
            .summary("❌ Tide - Task Failed")
            .body(&format!(
                "Task '{}' (group: {}) failed:\n{}",
                task_name, group_name, error_preview
            ))
            .icon("dialog-error")
            .timeout(Timeout::Milliseconds(8000))
            .show()?;

        Ok(())
    }

    /// Send a notification that sudo authentication is required
    pub fn notify_sudo_required(&self) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        Notification::new()
            .summary("🔐 Tide - Sudo Password Required")
            .body("Some tasks require sudo privileges.\nPlease check your terminal to enter your password.")
            .icon("dialog-password")
            .timeout(Timeout::Milliseconds(10000))
            .show()?;

        Ok(())
    }

    /// Send a notification that all tasks completed successfully
    pub fn notify_all_tasks_complete(
        &self,
        success_count: usize,
        total_duration_secs: u64,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        Notification::new()
            .summary("✅ Tide - All Tasks Complete")
            .body(&format!(
                "{} tasks completed successfully in {} seconds.",
                success_count, total_duration_secs
            ))
            .icon("emblem-default")
            .timeout(Timeout::Milliseconds(5000))
            .show()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_manager_disabled() {
        let manager = NotificationManager::new(false);
        // Should not error even when disabled
        assert!(
            manager
                .notify_interactive_input_detected("test", "group")
                .is_ok()
        );
    }
}
