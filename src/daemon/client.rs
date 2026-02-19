use super::protocol::{
    ArchivedResponse, MessageFrame, Request, Response, ServiceConfig, ServiceInfo,
};
use crate::clienv;
use anyhow::{anyhow, Result};
use lib_daemon_core::{spawn_background, SpawnConfig};
use std::path::PathBuf;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, trace};

/// Default timeout for IPC operations
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Maximum time to wait for daemon to start
const DAEMON_START_TIMEOUT: Duration = Duration::from_secs(5);

/// Interval for checking daemon startup
const DAEMON_START_CHECK_INTERVAL: Duration = Duration::from_millis(100);

/// Client for communicating with the ADI daemon
pub struct DaemonClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl DaemonClient {
    /// Create a new daemon client with default socket path
    pub fn new() -> Self {
        Self {
            socket_path: clienv::daemon_socket_path(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Create a daemon client with custom socket path
    pub fn with_socket_path(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            timeout: DEFAULT_TIMEOUT,
        }
    }

    /// Set timeout for operations
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if daemon socket exists (daemon may be running)
    pub fn socket_exists(&self) -> bool {
        self.socket_path.exists()
    }

    /// Check if daemon is running by sending a ping
    pub async fn is_running(&self) -> bool {
        if !self.socket_exists() {
            return false;
        }
        self.ping().await.is_ok()
    }

    /// Ping the daemon
    pub async fn ping(&self) -> Result<(u64, String)> {
        let response = self.request(&Request::Ping).await?;
        match response {
            Response::Pong {
                uptime_secs,
                version,
            } => Ok((uptime_secs, version)),
            Response::Error { message } => Err(anyhow!("Daemon error: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Shutdown the daemon
    pub async fn shutdown(&self, graceful: bool) -> Result<()> {
        let response = self.request(&Request::Shutdown { graceful }).await?;
        match response {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(anyhow!("Shutdown failed: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Start a service
    pub async fn start_service(&self, name: &str, config: Option<ServiceConfig>) -> Result<()> {
        let response = self
            .request(&Request::StartService {
                name: name.to_string(),
                config,
            })
            .await?;
        match response {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(anyhow!("Failed to start service: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Stop a service
    pub async fn stop_service(&self, name: &str, force: bool) -> Result<()> {
        let response = self
            .request(&Request::StopService {
                name: name.to_string(),
                force,
            })
            .await?;
        match response {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(anyhow!("Failed to stop service: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Restart a service
    pub async fn restart_service(&self, name: &str) -> Result<()> {
        let response = self
            .request(&Request::RestartService {
                name: name.to_string(),
            })
            .await?;
        match response {
            Response::Ok => Ok(()),
            Response::Error { message } => Err(anyhow!("Failed to restart service: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// List all services
    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        let response = self.request(&Request::ListServices).await?;
        match response {
            Response::Services { list } => Ok(list),
            Response::Error { message } => Err(anyhow!("Failed to list services: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Get service logs
    pub async fn service_logs(&self, name: &str, lines: usize) -> Result<Vec<String>> {
        let response = self
            .request(&Request::ServiceLogs {
                name: name.to_string(),
                lines,
                follow: false,
            })
            .await?;
        match response {
            Response::Logs { lines } => Ok(lines),
            Response::Error { message } => Err(anyhow!("Failed to get logs: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Execute a command as regular user (adi)
    pub async fn run(&self, command: &str, args: &[String]) -> Result<CommandOutput> {
        let response = self
            .request(&Request::Run {
                command: command.to_string(),
                args: args.to_vec(),
            })
            .await?;
        match response {
            Response::CommandResult {
                exit_code,
                stdout,
                stderr,
            } => Ok(CommandOutput {
                exit_code,
                stdout,
                stderr,
            }),
            Response::Error { message } => Err(anyhow!("Command failed: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Execute a command as privileged user (adi-root)
    pub async fn sudo_run(
        &self,
        command: &str,
        args: &[String],
        reason: &str,
    ) -> Result<CommandOutput> {
        let response = self
            .request(&Request::SudoRun {
                command: command.to_string(),
                args: args.to_vec(),
                reason: reason.to_string(),
            })
            .await?;
        match response {
            Response::CommandResult {
                exit_code,
                stdout,
                stderr,
            } => Ok(CommandOutput {
                exit_code,
                stdout,
                stderr,
            }),
            Response::SudoDenied { reason } => Err(anyhow!("Sudo denied: {}", reason)),
            Response::Error { message } => Err(anyhow!("Command failed: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Bind a privileged port to a high port
    pub async fn bind_port(&self, port: u16, target_port: u16) -> Result<()> {
        let response = self
            .request(&Request::BindPort { port, target_port })
            .await?;
        match response {
            Response::Ok => Ok(()),
            Response::SudoDenied { reason } => Err(anyhow!("Port bind denied: {}", reason)),
            Response::Error { message } => Err(anyhow!("Port bind failed: {}", message)),
            _ => Err(anyhow!("Unexpected response")),
        }
    }

    /// Ensure daemon is running, auto-starting if needed
    pub async fn ensure_running(&self) -> Result<()> {
        if self.is_running().await {
            debug!("Daemon already running");
            return Ok(());
        }

        info!("Starting daemon...");
        start_daemon()?;

        // Wait for socket to appear
        let start = std::time::Instant::now();
        while start.elapsed() < DAEMON_START_TIMEOUT {
            if self.socket_exists() {
                // Socket exists, try to ping
                if self.ping().await.is_ok() {
                    info!("Daemon started successfully");
                    return Ok(());
                }
            }
            tokio::time::sleep(DAEMON_START_CHECK_INTERVAL).await;
        }

        Err(anyhow!(
            "Daemon failed to start within {:?}",
            DAEMON_START_TIMEOUT
        ))
    }

    /// Send a request and receive a response
    async fn request(&self, request: &Request) -> Result<Response> {
        let result = tokio::time::timeout(self.timeout, self.request_inner(request)).await;

        match result {
            Ok(inner_result) => inner_result,
            Err(_) => Err(anyhow!(
                "Daemon request timed out after {:?}",
                self.timeout
            )),
        }
    }

    async fn request_inner(&self, request: &Request) -> Result<Response> {
        // Connect to socket
        #[cfg(unix)]
        let mut stream = tokio::net::UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| anyhow!("Failed to connect to daemon: {}", e))?;

        #[cfg(not(unix))]
        let mut stream = {
            // On non-Unix, fall back to TCP
            let port = clienv::daemon_tcp_port();
            tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
                .await
                .map_err(|e| anyhow!("Failed to connect to daemon: {}", e))?
        };

        trace!("Connected to daemon socket");

        // Encode request
        let request_bytes = MessageFrame::encode_request(request)
            .map_err(|e| anyhow!("Failed to encode request: {}", e))?;

        // Send request
        stream.write_all(&request_bytes).await?;
        stream.flush().await?;
        trace!("Sent request to daemon");

        // Read response length
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = MessageFrame::read_length(&len_buf);
        trace!("Response length: {} bytes", len);

        // Read response bytes
        let mut response_buf = vec![0u8; len];
        stream.read_exact(&mut response_buf).await?;

        // Zero-copy access to response
        let archived = rkyv::access::<ArchivedResponse, rkyv::rancor::Error>(&response_buf)
            .map_err(|e| anyhow!("Failed to deserialize response: {}", e))?;

        // Convert archived response to owned response
        let response = deserialize_response(archived)?;

        Ok(response)
    }
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from a command execution
#[derive(Debug, Clone)]
pub struct CommandOutput {
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: Vec<u8>,
    /// Standard error
    pub stderr: Vec<u8>,
}

impl CommandOutput {
    /// Check if command succeeded (exit code 0)
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get stdout as string (lossy UTF-8)
    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).to_string()
    }

    /// Get stderr as string (lossy UTF-8)
    pub fn stderr_str(&self) -> String {
        String::from_utf8_lossy(&self.stderr).to_string()
    }
}

/// Start the daemon process in the background
fn start_daemon() -> Result<u32> {
    let exe = std::env::current_exe()?;
    let log_path = clienv::daemon_log_path();
    let pid_path = clienv::daemon_pid_path();

    // Ensure log directory exists
    if let Some(parent) = log_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let config = SpawnConfig::new(exe.display().to_string())
        .args(["daemon", "run"])
        .stdout(log_path.display().to_string())
        .stderr(log_path.display().to_string())
        .pid_file(pid_path.display().to_string())
        .env("RUST_LOG", "info");

    let pid = spawn_background(&config)?;
    debug!("Daemon spawned with PID {}", pid);

    Ok(pid)
}

/// Deserialize archived response to owned response
fn deserialize_response(archived: &ArchivedResponse) -> Result<Response> {
    use super::protocol::ArchivedServiceInfo;

    match archived {
        ArchivedResponse::Pong {
            uptime_secs,
            version,
        } => Ok(Response::Pong {
            uptime_secs: (*uptime_secs).into(),
            version: version.to_string(),
        }),
        ArchivedResponse::Ok => Ok(Response::Ok),
        ArchivedResponse::Error { message } => Ok(Response::Error {
            message: message.to_string(),
        }),
        ArchivedResponse::Services { list } => {
            let services: Vec<ServiceInfo> = list
                .iter()
                .map(|s: &ArchivedServiceInfo| deserialize_service_info(s))
                .collect();
            Ok(Response::Services { list: services })
        }
        ArchivedResponse::Logs { lines } => {
            let logs: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            Ok(Response::Logs { lines: logs })
        }
        ArchivedResponse::LogLine { line } => Ok(Response::LogLine {
            line: line.to_string(),
        }),
        ArchivedResponse::StreamEnd => Ok(Response::StreamEnd),
        ArchivedResponse::CommandResult {
            exit_code,
            stdout,
            stderr,
        } => Ok(Response::CommandResult {
            exit_code: (*exit_code).into(),
            stdout: stdout.to_vec(),
            stderr: stderr.to_vec(),
        }),
        ArchivedResponse::SudoDenied { reason } => Ok(Response::SudoDenied {
            reason: reason.to_string(),
        }),
    }
}

fn deserialize_service_info(
    archived: &super::protocol::ArchivedServiceInfo,
) -> ServiceInfo {
    use super::protocol::{ArchivedServiceState, ServiceState};

    let state = match archived.state {
        ArchivedServiceState::Starting => ServiceState::Starting,
        ArchivedServiceState::Running => ServiceState::Running,
        ArchivedServiceState::Stopping => ServiceState::Stopping,
        ArchivedServiceState::Stopped => ServiceState::Stopped,
        ArchivedServiceState::Failed => ServiceState::Failed,
    };

    ServiceInfo {
        name: archived.name.to_string(),
        state,
        pid: archived.pid.as_ref().map(|p| (*p).into()),
        uptime_secs: archived.uptime_secs.as_ref().map(|u| (*u).into()),
        restarts: archived.restarts.into(),
        last_error: archived.last_error.as_ref().map(|s| s.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_output() {
        let output = CommandOutput {
            exit_code: 0,
            stdout: b"hello".to_vec(),
            stderr: b"".to_vec(),
        };

        assert!(output.success());
        assert_eq!(output.stdout_str(), "hello");
        assert_eq!(output.stderr_str(), "");
    }

    #[test]
    fn test_command_output_failure() {
        let output = CommandOutput {
            exit_code: 1,
            stdout: b"".to_vec(),
            stderr: b"error message".to_vec(),
        };

        assert!(!output.success());
        assert_eq!(output.stderr_str(), "error message");
    }
}
