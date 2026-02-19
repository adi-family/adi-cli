//! IPC Protocol types using rkyv for zero-copy serialization
//!
//! All request/response types use `#[derive(Archive, Serialize, Deserialize)]`
//! for zero-copy deserialization. The bytes ARE the struct - no parsing needed.

use rkyv::{Archive, Deserialize, Serialize};

/// IPC request from client to daemon
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub enum Request {
    // Daemon lifecycle
    /// Check if daemon is alive
    Ping,
    /// Shutdown the daemon
    Shutdown {
        /// Wait for services to stop gracefully
        graceful: bool,
    },

    // Service management
    /// Start a service
    StartService {
        name: String,
        config: Option<ServiceConfig>,
    },
    /// Stop a service
    StopService {
        name: String,
        /// Force kill (SIGKILL) instead of graceful (SIGTERM)
        force: bool,
    },
    /// Restart a service
    RestartService { name: String },
    /// List all services
    ListServices,
    /// Get service logs
    ServiceLogs {
        name: String,
        /// Number of lines to return
        lines: usize,
        /// Stream logs continuously
        follow: bool,
    },

    // Command execution
    /// Execute command as regular user (adi)
    Run { command: String, args: Vec<String> },
    /// Execute command as privileged user (adi-root)
    SudoRun {
        command: String,
        args: Vec<String>,
        /// Human-readable reason for the privileged operation
        reason: String,
    },

    // Convenience privileged operations
    /// Bind a privileged port to a high port
    BindPort {
        /// Privileged port (< 1024)
        port: u16,
        /// Target high port
        target_port: u16,
    },
}

/// IPC response from daemon to client
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub enum Response {
    /// Response to Ping
    Pong { uptime_secs: u64, version: String },
    /// Generic success
    Ok,
    /// Error occurred
    Error { message: String },
    /// List of services
    Services { list: Vec<ServiceInfo> },
    /// Log lines
    Logs { lines: Vec<String> },
    /// Single log line (for streaming)
    LogLine { line: String },
    /// End of stream
    StreamEnd,
    /// Command execution result
    CommandResult {
        exit_code: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    /// Privileged command denied
    SudoDenied { reason: String },
}

/// Service information
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ServiceInfo {
    /// Service name (e.g., "hive", "indexer")
    pub name: String,
    /// Current state
    pub state: ServiceState,
    /// Process ID if running
    pub pid: Option<u32>,
    /// Uptime in seconds if running
    pub uptime_secs: Option<u64>,
    /// Number of restarts since daemon started
    pub restarts: u32,
    /// Last error message if failed
    pub last_error: Option<String>,
}

impl ServiceInfo {
    /// Create new service info
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: ServiceState::Stopped,
            pid: None,
            uptime_secs: None,
            restarts: 0,
            last_error: None,
        }
    }
}

/// Service state
#[derive(Archive, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub enum ServiceState {
    /// Service is starting up
    Starting,
    /// Service is running
    Running,
    /// Service is stopping
    Stopping,
    /// Service is stopped
    Stopped,
    /// Service failed (check last_error)
    Failed,
}

impl ServiceState {
    /// Check if service is in a running state
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    /// Check if service is in a stopped state
    pub fn is_stopped(&self) -> bool {
        matches!(self, ServiceState::Stopped | ServiceState::Failed)
    }

    /// Get human-readable state name
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceState::Starting => "starting",
            ServiceState::Running => "running",
            ServiceState::Stopping => "stopping",
            ServiceState::Stopped => "stopped",
            ServiceState::Failed => "failed",
        }
    }
}

/// Service configuration for starting a service
#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ServiceConfig {
    /// Command to execute
    pub command: String,
    /// Command arguments
    pub args: Vec<String>,
    /// Environment variables
    pub env: Vec<(String, String)>,
    /// Working directory (as string path)
    pub working_dir: Option<String>,
    /// Restart on failure
    pub restart_on_failure: bool,
    /// Maximum restart attempts
    pub max_restarts: u32,
    /// Run as privileged user (adi-root)
    pub privileged: bool,
}

impl ServiceConfig {
    /// Create a new service config
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: Vec::new(),
            working_dir: None,
            restart_on_failure: true,
            max_restarts: 3,
            privileged: false,
        }
    }

    /// Set command arguments
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add environment variable
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Set working directory
    pub fn working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Set restart on failure
    pub fn restart_on_failure(mut self, restart: bool) -> Self {
        self.restart_on_failure = restart;
        self
    }

    /// Set max restarts
    pub fn max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

    /// Set privileged mode
    pub fn privileged(mut self, privileged: bool) -> Self {
        self.privileged = privileged;
        self
    }
}

/// Message frame for wire protocol
///
/// Format: [4-byte length (little-endian)][rkyv bytes]
pub struct MessageFrame;

impl MessageFrame {
    /// Encode a request to bytes with length prefix
    pub fn encode_request(request: &Request) -> Result<Vec<u8>, rkyv::rancor::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(request)?;
        let len = bytes.len() as u32;
        let mut result = Vec::with_capacity(4 + bytes.len());
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&bytes);
        Ok(result)
    }

    /// Encode a response to bytes with length prefix
    pub fn encode_response(response: &Response) -> Result<Vec<u8>, rkyv::rancor::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(response)?;
        let len = bytes.len() as u32;
        let mut result = Vec::with_capacity(4 + bytes.len());
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&bytes);
        Ok(result)
    }

    /// Read length prefix from buffer
    pub fn read_length(buf: &[u8; 4]) -> usize {
        u32::from_le_bytes(*buf) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_roundtrip() {
        let request = Request::Ping;
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&request).unwrap();
        let archived = rkyv::access::<ArchivedRequest, rkyv::rancor::Error>(&bytes).unwrap();
        assert!(matches!(archived, ArchivedRequest::Ping));
    }

    #[test]
    fn test_response_roundtrip() {
        let response = Response::Pong {
            uptime_secs: 3600,
            version: "1.0.0".to_string(),
        };
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&response).unwrap();
        let archived = rkyv::access::<ArchivedResponse, rkyv::rancor::Error>(&bytes).unwrap();

        if let ArchivedResponse::Pong {
            uptime_secs,
            version,
        } = archived
        {
            assert_eq!(*uptime_secs, 3600);
            assert_eq!(version.as_str(), "1.0.0");
        } else {
            panic!("Expected Pong response");
        }
    }

    #[test]
    fn test_service_state() {
        assert!(ServiceState::Running.is_running());
        assert!(!ServiceState::Stopped.is_running());
        assert!(ServiceState::Stopped.is_stopped());
        assert!(ServiceState::Failed.is_stopped());
        assert!(!ServiceState::Running.is_stopped());
    }

    #[test]
    fn test_service_config_builder() {
        let config = ServiceConfig::new("my-service")
            .args(["--flag", "value"])
            .env("RUST_LOG", "info")
            .working_dir("/var/lib/service")
            .restart_on_failure(true)
            .max_restarts(5)
            .privileged(false);

        assert_eq!(config.command, "my-service");
        assert_eq!(config.args, vec!["--flag", "value"]);
        assert!(config
            .env
            .iter()
            .any(|(k, v)| k == "RUST_LOG" && v == "info"));
        assert_eq!(config.working_dir, Some("/var/lib/service".to_string()));
        assert!(config.restart_on_failure);
        assert_eq!(config.max_restarts, 5);
        assert!(!config.privileged);
    }
}
