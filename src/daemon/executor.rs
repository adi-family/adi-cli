use crate::clienv;
use anyhow::Result;
use std::process::Output;
use tokio::process::Command;
use tracing::{debug, info};

/// Runs commands as either `adi` (unprivileged) or `adi-root` (sudo) users
pub struct CommandExecutor {
    regular_user: String,
    privileged_user: String,
}

impl CommandExecutor {
    pub fn new() -> Self {
        Self {
            regular_user: clienv::daemon_user(),
            privileged_user: clienv::daemon_root_user(),
        }
    }

    /// Runs with `adi` user privileges (no sudo access).
    pub async fn run(&self, cmd: &str, args: &[String]) -> Result<Output> {
        debug!("Running command as {}: {} {:?}", self.regular_user, cmd, args);

        #[cfg(unix)]
        {
            let output = Command::new("sudo")
                .args(["-u", &self.regular_user, cmd])
                .args(args)
                .output()
                .await?;

            debug!(
                "Command finished with exit code: {:?}",
                output.status.code()
            );
            Ok(output)
        }

        #[cfg(not(unix))]
        {
            // On Windows, run directly (no sudo equivalent)
            let output = Command::new(cmd).args(args).output().await?;
            Ok(output)
        }
    }

    /// Runs with root privileges via `adi-root` user (NOPASSWD sudo).
    /// Only call after validating the plugin has permission for this command.
    pub async fn sudo_run(&self, cmd: &str, args: &[String]) -> Result<Output> {
        info!(
            "Running privileged command as {}: {} {:?}",
            self.privileged_user, cmd, args
        );

        #[cfg(unix)]
        {
            // sudo -u adi-root sudo <cmd> <args>
            // First sudo switches to adi-root, second sudo executes as root
            let output = Command::new("sudo")
                .args(["-u", &self.privileged_user, "sudo", cmd])
                .args(args)
                .output()
                .await?;

            debug!(
                "Privileged command finished with exit code: {:?}",
                output.status.code()
            );
            Ok(output)
        }

        #[cfg(not(unix))]
        {
            // On Windows, privileged execution requires different approach
            warn!("Privileged execution not fully supported on Windows");
            let output = Command::new(cmd).args(args).output().await?;
            Ok(output)
        }
    }
}

impl Default for CommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = CommandExecutor::new();
        assert_eq!(executor.regular_user, clienv::daemon_user());
        assert_eq!(executor.privileged_user, clienv::daemon_root_user());
    }
}
