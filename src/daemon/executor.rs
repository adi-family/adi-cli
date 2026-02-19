use crate::clienv;
use anyhow::Result;
use std::process::Output;
use tokio::process::Command;
use tracing::{debug, info};

/// Command executor with privilege isolation
pub struct CommandExecutor {
    /// Regular user for normal operations
    regular_user: String,
    /// Privileged user for root operations
    privileged_user: String,
}

impl CommandExecutor {
    /// Create a new command executor
    pub fn new() -> Self {
        Self {
            regular_user: clienv::daemon_user(),
            privileged_user: clienv::daemon_root_user(),
        }
    }

    /// Execute command as regular user (adi)
    ///
    /// The command runs with the privileges of the `adi` user,
    /// which has no sudo access.
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

    /// Execute command as privileged user (adi-root)
    ///
    /// The command runs with root privileges via the `adi-root` user,
    /// which has NOPASSWD sudo access.
    ///
    /// # Security
    ///
    /// This method should only be called after validating that the
    /// requesting plugin has permission for the specific command.
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

    /// Bind a privileged port (< 1024) to a high port
    ///
    /// Uses platform-specific methods:
    /// - Linux: iptables NAT rules
    /// - macOS: pfctl (packet filter)
    pub async fn bind_port(&self, port: u16, target_port: u16) -> Result<()> {
        info!(
            "Binding privileged port {} to target port {}",
            port, target_port
        );

        if port >= 1024 {
            // Not a privileged port, no action needed
            debug!("Port {} is not privileged, no binding needed", port);
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            self.bind_port_linux(port, target_port).await
        }

        #[cfg(target_os = "macos")]
        {
            self.bind_port_macos(port, target_port).await
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            warn!("Port binding not supported on this platform");
            Ok(())
        }
    }

    /// Linux port binding using iptables
    #[cfg(target_os = "linux")]
    async fn bind_port_linux(&self, port: u16, target_port: u16) -> Result<()> {
        let output = self
            .sudo_run(
                "iptables",
                &[
                    "-t".to_string(),
                    "nat".to_string(),
                    "-A".to_string(),
                    "PREROUTING".to_string(),
                    "-p".to_string(),
                    "tcp".to_string(),
                    "--dport".to_string(),
                    port.to_string(),
                    "-j".to_string(),
                    "REDIRECT".to_string(),
                    "--to-port".to_string(),
                    target_port.to_string(),
                ],
            )
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("iptables failed: {}", stderr);
        }

        info!("Port {} redirected to {} via iptables", port, target_port);
        Ok(())
    }

    /// macOS port binding using pfctl
    #[cfg(target_os = "macos")]
    async fn bind_port_macos(&self, port: u16, target_port: u16) -> Result<()> {
        // Create pf rule
        let rule = format!(
            "rdr pass on lo0 inet proto tcp from any to any port {} -> 127.0.0.1 port {}",
            port, target_port
        );

        // Write rule to temp file
        let rule_path = format!("/tmp/adi-pf-{}.conf", port);
        tokio::fs::write(&rule_path, &rule).await?;

        // Load rule with pfctl
        let output = self
            .sudo_run("pfctl", &["-f".to_string(), rule_path.clone()])
            .await?;

        // Cleanup temp file
        let _ = tokio::fs::remove_file(&rule_path).await;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("pfctl failed: {}", stderr);
        }

        // Enable pf if not already enabled
        let _ = self.sudo_run("pfctl", &["-e".to_string()]).await;

        info!("Port {} redirected to {} via pfctl", port, target_port);
        Ok(())
    }

    /// Unbind a previously bound port
    pub async fn unbind_port(&self, port: u16) -> Result<()> {
        info!("Unbinding privileged port {}", port);

        #[cfg(target_os = "linux")]
        {
            // Remove iptables rule (best effort)
            let _ = self
                .sudo_run(
                    "iptables",
                    &[
                        "-t".to_string(),
                        "nat".to_string(),
                        "-D".to_string(),
                        "PREROUTING".to_string(),
                        "-p".to_string(),
                        "tcp".to_string(),
                        "--dport".to_string(),
                        port.to_string(),
                        "-j".to_string(),
                        "REDIRECT".to_string(),
                    ],
                )
                .await;
        }

        #[cfg(target_os = "macos")]
        {
            // pfctl rules are session-based, will be removed on reboot
            // For explicit removal, we'd need to reload pf.conf without the rule
            debug!("macOS pfctl rules require manual cleanup or reboot");
        }

        Ok(())
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
