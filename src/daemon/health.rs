use super::log_buffer::LogBuffer;
use super::protocol::ServiceState;
use super::services::{ManagedService, ServiceManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(5);

pub struct HealthManager {
    services: Arc<RwLock<HashMap<String, ManagedService>>>,
    log_buffer: Arc<LogBuffer>,
    check_interval: Duration,
}

impl HealthManager {
    pub fn new(service_manager: &ServiceManager) -> Self {
        Self {
            services: service_manager.services_ref(),
            log_buffer: Arc::clone(service_manager.log_buffer()),
            check_interval: DEFAULT_CHECK_INTERVAL,
        }
    }

    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Spawns as a background task to monitor service health.
    pub async fn run(&self) {
        info!(
            "Health manager started (interval: {:?})",
            self.check_interval
        );

        let mut interval = tokio::time::interval(self.check_interval);

        loop {
            interval.tick().await;
            self.check_all().await;
        }
    }

    async fn check_all(&self) {
        // Collect names of running services, then check each one while
        // holding a write lock so we can call try_wait() to reap zombies.
        let running_names: Vec<String> = {
            let services = self.services.read().await;
            services
                .iter()
                .filter(|(_, s)| s.state == ServiceState::Running)
                .map(|(name, _)| name.clone())
                .collect()
        };

        for name in running_names {
            let (alive, pid, restart_on_failure, max_restarts) = {
                let mut services = self.services.write().await;
                let Some(service) = services.get_mut(&name) else {
                    continue;
                };
                let restart_on_failure = service.config.restart_on_failure;
                let max_restarts = service.config.max_restarts;
                let pid = service.pid();

                // Prefer try_wait() on owned Child handle -- this both detects
                // exit and reaps zombies so they don't linger in the process table.
                let alive = if let Some(ref mut child) = service.process {
                    match child.try_wait() {
                        Ok(Some(_exit_status)) => false, // exited (zombie reaped)
                        Ok(None) => true,                // still running
                        Err(_) => false,                 // error querying, treat as dead
                    }
                } else if let Some(pid) = pid {
                    // Fallback to PID-based check (includes zombie detection)
                    lib_daemon_core::is_process_running(pid)
                } else {
                    false
                };

                (alive, pid, restart_on_failure, max_restarts)
            };

            if !alive {
                if let Some(pid) = pid {
                    warn!("Service '{}' (PID {}) has died unexpectedly", name, pid);
                } else {
                    warn!("Service '{}' has no PID, marking as failed", name);
                }
                self.handle_service_death(&name, restart_on_failure, max_restarts)
                    .await;
            } else {
                debug!("Service '{}' (PID {:?}) is healthy", name, pid);
            }
        }
    }

    async fn handle_service_death(&self, name: &str, restart_on_failure: bool, max_restarts: u32) {
        let mut services = self.services.write().await;

        if let Some(service) = services.get_mut(name) {
            if restart_on_failure && service.restarts < max_restarts {
                info!(
                    "Restarting service '{}' (attempt {}/{})",
                    name,
                    service.restarts + 1,
                    max_restarts
                );

                service.state = ServiceState::Starting;
                service.restarts += 1;
                service.process = None;
                service.started_at = None;

                let config = service.config.clone();
                drop(services);

                if let Err(e) = self.restart_service(name, &config).await {
                    error!("Failed to restart service '{}': {}", name, e);
                    self.mark_failed(name, &e.to_string()).await;
                }
            } else {
                service.state = ServiceState::Failed;
                service.last_error = Some("Process died and max restarts exceeded".to_string());
                service.process = None;

                error!(
                    "Service '{}' failed after {} restarts",
                    name, service.restarts
                );
            }
        }
    }

    async fn restart_service(
        &self,
        name: &str,
        config: &super::protocol::ServiceConfig,
    ) -> anyhow::Result<()> {
        use std::process::Stdio;
        use tokio::process::Command;

        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args);

        for (key, value) in &config.env {
            cmd.env(key, value);
        }

        if let Some(ref dir) = config.working_dir {
            cmd.current_dir(dir);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        // Capture stdout/stderr into log buffer
        if let Some(stdout) = child.stdout.take() {
            let buf = Arc::clone(&self.log_buffer);
            let svc = name.to_string();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    buf.push(&svc, line);
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            let buf = Arc::clone(&self.log_buffer);
            let svc = name.to_string();
            tokio::spawn(async move {
                let mut lines = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    buf.push(&svc, line);
                }
            });
        }

        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            let pid = child.id();
            info!("Service '{}' restarted with PID {:?}", name, pid);

            service.process = Some(child);
            service.state = ServiceState::Running;
            service.started_at = Some(std::time::Instant::now());
            service.last_error = None;
        }

        Ok(())
    }

    async fn mark_failed(&self, name: &str, error: &str) {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            service.state = ServiceState::Failed;
            service.last_error = Some(error.to_string());
            service.process = None;
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub total: usize,
    pub running: usize,
    pub stopped: usize,
    pub failed: usize,
    /// Services that need attention (failed or restarting frequently)
    pub unhealthy: Vec<String>,
}

impl HealthStatus {
    pub async fn from_services(services: &Arc<RwLock<HashMap<String, ManagedService>>>) -> Self {
        let services = services.read().await;

        let mut status = HealthStatus {
            total: services.len(),
            running: 0,
            stopped: 0,
            failed: 0,
            unhealthy: Vec::new(),
        };

        for (name, service) in services.iter() {
            match service.state {
                ServiceState::Running => status.running += 1,
                ServiceState::Stopped => status.stopped += 1,
                ServiceState::Failed => {
                    status.failed += 1;
                    status.unhealthy.push(name.clone());
                }
                ServiceState::Starting | ServiceState::Stopping => {
                    // Transitional states
                }
            }

            // Flag services with many restarts as unhealthy
            if service.restarts >= 2 && !status.unhealthy.contains(name) {
                status.unhealthy.push(name.clone());
            }
        }

        status
    }

    pub fn is_healthy(&self) -> bool {
        self.failed == 0 && self.unhealthy.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus {
            total: 3,
            running: 3,
            stopped: 0,
            failed: 0,
            unhealthy: Vec::new(),
        };

        assert!(status.is_healthy());
    }

    #[test]
    fn test_health_status_unhealthy() {
        let status = HealthStatus {
            total: 3,
            running: 2,
            stopped: 0,
            failed: 1,
            unhealthy: vec!["failed-service".to_string()],
        };

        assert!(!status.is_healthy());
    }
}
