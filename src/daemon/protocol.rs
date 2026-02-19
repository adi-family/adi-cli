use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub enum Request {
    Ping,
    Shutdown {
        graceful: bool,
    },

    StartService {
        name: String,
        config: Option<ServiceConfig>,
    },
    StopService {
        name: String,
        /// SIGKILL instead of graceful SIGTERM
        force: bool,
    },
    RestartService {
        name: String,
    },
    ListServices,
    ServiceLogs {
        name: String,
        lines: usize,
        follow: bool,
    },

    /// Runs as regular user (adi)
    Run {
        command: String,
        args: Vec<String>,
    },
    /// Runs as privileged user (adi-root)
    SudoRun {
        command: String,
        args: Vec<String>,
        reason: String,
    },
}

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub enum Response {
    Pong {
        uptime_secs: u64,
        version: String,
    },
    Ok,
    Error {
        message: String,
    },
    Services {
        list: Vec<ServiceInfo>,
    },
    Logs {
        lines: Vec<String>,
    },
    /// For streaming mode
    LogLine {
        line: String,
    },
    StreamEnd,
    CommandResult {
        exit_code: i32,
        stdout: Vec<u8>,
        stderr: Vec<u8>,
    },
    SudoDenied {
        reason: String,
    },
}

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ServiceInfo {
    pub name: String,
    pub state: ServiceState,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    /// Number of restarts since daemon started
    pub restarts: u32,
    pub last_error: Option<String>,
}

impl ServiceInfo {
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

#[derive(Archive, Deserialize, Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[rkyv(derive(Debug))]
pub enum ServiceState {
    Starting,
    Running,
    Stopping,
    Stopped,
    /// Check last_error for details
    Failed,
}

impl ServiceState {
    pub fn is_running(&self) -> bool {
        matches!(self, ServiceState::Running)
    }

    pub fn is_stopped(&self) -> bool {
        matches!(self, ServiceState::Stopped | ServiceState::Failed)
    }

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

#[derive(Archive, Deserialize, Serialize, Debug, Clone)]
#[rkyv(derive(Debug))]
pub struct ServiceConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
    /// String path, not PathBuf (for serialization)
    pub working_dir: Option<String>,
    pub restart_on_failure: bool,
    pub max_restarts: u32,
    /// Runs as adi-root instead of adi
    pub privileged: bool,
}

impl ServiceConfig {
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

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub fn working_dir(mut self, dir: impl Into<String>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    pub fn restart_on_failure(mut self, restart: bool) -> Self {
        self.restart_on_failure = restart;
        self
    }

    pub fn max_restarts(mut self, max: u32) -> Self {
        self.max_restarts = max;
        self
    }

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
    pub fn encode_request(request: &Request) -> Result<Vec<u8>, rkyv::rancor::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(request)?;
        let len = bytes.len() as u32;
        let mut result = Vec::with_capacity(4 + bytes.len());
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&bytes);
        Ok(result)
    }

    pub fn encode_response(response: &Response) -> Result<Vec<u8>, rkyv::rancor::Error> {
        let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(response)?;
        let len = bytes.len() as u32;
        let mut result = Vec::with_capacity(4 + bytes.len());
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(&bytes);
        Ok(result)
    }

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
