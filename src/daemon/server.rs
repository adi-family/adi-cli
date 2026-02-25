use super::executor::CommandExecutor;
use super::health::HealthManager;
use super::log_buffer::LogBuffer;
use super::protocol::{ArchivedRequest, MessageFrame, Response};
use super::services::ServiceManager;
use crate::clienv;
use anyhow::Result;
use lib_daemon_core::{PidFile, ShutdownCoordinator, ShutdownHandle};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use tracing::{debug, error, info, trace, warn};

pub struct DaemonConfig {
    pub socket_path: std::path::PathBuf,
    pub pid_path: std::path::PathBuf,
    pub log_path: std::path::PathBuf,
    pub auto_start: Vec<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: clienv::daemon_socket_path(),
            pid_path: clienv::daemon_pid_path(),
            log_path: clienv::daemon_log_path(),
            auto_start: Vec::new(),
        }
    }
}

pub struct DaemonServer {
    config: DaemonConfig,
    services: Arc<ServiceManager>,
    executor: Arc<CommandExecutor>,
    started_at: Instant,
    version: String,
    shutdown_handle: Option<ShutdownHandle>,
}

impl DaemonServer {
    pub async fn new(mut config: DaemonConfig) -> Self {
        let log_buffer = Arc::new(LogBuffer::default());
        let mut manager = ServiceManager::new(Arc::clone(&log_buffer));
        if let Err(e) = manager.discover_plugins().await {
            warn!("Failed to discover plugin daemon services: {}", e);
        }

        // Extend auto_start with any services marked auto_start=true in their manifests
        let discovered = manager.auto_start_names();
        for name in discovered {
            if !config.auto_start.contains(&name) {
                info!("Scheduling auto-start for discovered service: {}", name);
                config.auto_start.push(name);
            }
        }

        Self {
            config,
            services: Arc::new(manager),
            executor: Arc::new(CommandExecutor::new()),
            started_at: Instant::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            shutdown_handle: None,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        info!("ADI daemon starting...");

        let pid_file = PidFile::new(&self.config.pid_path);
        if let Some(pid) = pid_file.is_running()? {
            anyhow::bail!("Daemon already running with PID {}", pid);
        }
        drop(pid_file);

        let mut pid_file = PidFile::new(&self.config.pid_path);
        pid_file.write()?;
        info!("PID file written: {}", self.config.pid_path.display());

        if self.config.socket_path.exists() {
            std::fs::remove_file(&self.config.socket_path)?;
        }

        if let Some(parent) = self.config.socket_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        #[cfg(unix)]
        let listener = tokio::net::UnixListener::bind(&self.config.socket_path)?;

        #[cfg(not(unix))]
        let listener = {
            let port = clienv::daemon_tcp_port();
            tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?
        };

        info!(
            "IPC server listening on: {}",
            self.config.socket_path.display()
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600);
            std::fs::set_permissions(&self.config.socket_path, perms)?;
        }

        for name in &self.config.auto_start {
            info!("Auto-starting service: {}", name);
            if let Err(e) = self.services.start(name, None).await {
                warn!("Failed to auto-start '{}': {}", name, e);
            }
        }

        let health_manager = HealthManager::new(&self.services);
        tokio::spawn(async move {
            health_manager.run().await;
        });

        let mut shutdown = ShutdownCoordinator::new();
        self.shutdown_handle = Some(shutdown.handle());

        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm = signal(SignalKind::terminate())?;
            let mut sigint = signal(SignalKind::interrupt())?;
            let handle = shutdown.handle();

            tokio::spawn(async move {
                tokio::select! {
                    _ = sigterm.recv() => {
                        info!("Received SIGTERM");
                        handle.shutdown();
                    }
                    _ = sigint.recv() => {
                        info!("Received SIGINT");
                        handle.shutdown();
                    }
                }
            });
        }

        #[cfg(not(unix))]
        {
            let handle = shutdown.handle();
            tokio::spawn(async move {
                tokio::signal::ctrl_c().await.ok();
                info!("Received Ctrl+C");
                handle.shutdown();
            });
        }

        let server = Arc::new(self);
        info!("ADI daemon ready");

        loop {
            tokio::select! {
                conn = listener.accept() => {
                    match conn {
                        Ok((stream, _)) => {
                            let server = Arc::clone(&server);
                            tokio::spawn(async move {
                                if let Err(e) = server.handle_connection(stream).await {
                                    error!("Connection handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept error: {}", e);
                        }
                    }
                }
                _ = shutdown.wait() => {
                    info!("Shutdown signal received");
                    break;
                }
            }
        }

        info!("Stopping all services...");
        server.services.stop_all().await;

        if server.config.socket_path.exists() {
            std::fs::remove_file(&server.config.socket_path)?;
        }

        info!("ADI daemon stopped");
        Ok(())
    }

    #[cfg(unix)]
    async fn handle_connection(&self, mut stream: tokio::net::UnixStream) -> Result<()> {
        trace!("New connection accepted");

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = MessageFrame::read_length(&len_buf);
        trace!("Request length: {} bytes", len);

        let mut request_buf = vec![0u8; len];
        stream.read_exact(&mut request_buf).await?;

        let archived = rkyv::access::<ArchivedRequest, rkyv::rancor::Error>(&request_buf)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize request: {}", e))?;

        let response = self.handle_request(archived).await;

        let response_bytes = MessageFrame::encode_response(&response)
            .map_err(|e| anyhow::anyhow!("Failed to encode response: {}", e))?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        trace!("Response sent");
        Ok(())
    }

    #[cfg(not(unix))]
    async fn handle_connection(&self, mut stream: tokio::net::TcpStream) -> Result<()> {
        trace!("New connection accepted");

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let len = MessageFrame::read_length(&len_buf);

        let mut request_buf = vec![0u8; len];
        stream.read_exact(&mut request_buf).await?;

        let archived = rkyv::access::<ArchivedRequest, rkyv::rancor::Error>(&request_buf)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize request: {}", e))?;

        let response = self.handle_request(archived).await;

        let response_bytes = MessageFrame::encode_response(&response)
            .map_err(|e| anyhow::anyhow!("Failed to encode response: {}", e))?;
        stream.write_all(&response_bytes).await?;
        stream.flush().await?;

        Ok(())
    }

    async fn handle_request(&self, request: &ArchivedRequest) -> Response {
        match request {
            ArchivedRequest::Ping => {
                debug!("Handling: Ping");
                Response::Pong {
                    uptime_secs: self.started_at.elapsed().as_secs(),
                    version: self.version.clone(),
                }
            }

            ArchivedRequest::Shutdown { graceful } => {
                info!("Handling: Shutdown (graceful: {})", graceful);
                if let Some(handle) = &self.shutdown_handle {
                    handle.shutdown();
                }
                Response::Ok
            }

            ArchivedRequest::StartService { name, config } => {
                debug!("Handling: StartService({})", name);
                let config = config.as_ref().map(deserialize_service_config);
                match self.services.start(name.as_str(), config).await {
                    Ok(()) => Response::Ok,
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                }
            }

            ArchivedRequest::StopService { name, force } => {
                debug!("Handling: StopService({}, force: {})", name, force);
                match self.services.stop(name.as_str(), *force).await {
                    Ok(()) => Response::Ok,
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                }
            }

            ArchivedRequest::RestartService { name } => {
                debug!("Handling: RestartService({})", name);
                match self.services.restart(name.as_str()).await {
                    Ok(()) => Response::Ok,
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                }
            }

            ArchivedRequest::ListServices => {
                debug!("Handling: ListServices");
                let list = self.services.list().await;
                Response::Services { list }
            }

            ArchivedRequest::ServiceLogs { name, lines, follow: _ } => {
                let n: usize = (*lines).try_into().unwrap_or(100);
                debug!("Handling: ServiceLogs({}, lines: {})", name, n);
                let log_lines = self.services.log_buffer().tail(name.as_str(), n);
                Response::Logs { lines: log_lines }
            }

            ArchivedRequest::Run { command, args } => {
                debug!("Handling: Run({} {:?})", command, args);
                let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
                match self.executor.run(command.as_str(), &args).await {
                    Ok(output) => Response::CommandResult {
                        exit_code: output.status.code().unwrap_or(-1),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    },
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                }
            }

            ArchivedRequest::SudoRun { command, args, reason } => {
                info!("Handling: SudoRun({} {:?}) - {}", command, args, reason);
                let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

                match self.executor.sudo_run(command.as_str(), &args).await {
                    Ok(output) => Response::CommandResult {
                        exit_code: output.status.code().unwrap_or(-1),
                        stdout: output.stdout,
                        stderr: output.stderr,
                    },
                    Err(e) => Response::Error {
                        message: e.to_string(),
                    },
                }
            }
        }
    }
}

fn deserialize_service_config(
    archived: &super::protocol::ArchivedServiceConfig,
) -> super::protocol::ServiceConfig {
    let env: Vec<(String, String)> = archived
        .env
        .iter()
        .map(|pair| (pair.0.to_string(), pair.1.to_string()))
        .collect();

    super::protocol::ServiceConfig {
        command: archived.command.to_string(),
        args: archived.args.iter().map(|s| s.to_string()).collect(),
        env,
        working_dir: archived.working_dir.as_ref().map(|p| p.to_string()),
        restart_on_failure: archived.restart_on_failure,
        max_restarts: archived.max_restarts.into(),
        privileged: archived.privileged,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert!(config.socket_path.to_string_lossy().contains("daemon.sock"));
        assert!(config.pid_path.to_string_lossy().contains("daemon.pid"));
        assert!(config.auto_start.is_empty());
    }
}
