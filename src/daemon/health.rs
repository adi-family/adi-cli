use super::protocol::ServiceState;
use super::services::{ManagedService, ServiceManager};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Default health check interval
const DEFAULT_CHECK_INTERVAL: Duration = Duration::from_secs(5);

/// Health manager that monitors service processes
pub struct HealthManager {
    /// Reference to the service manager
    services: Arc<RwLock<HashMap<String, ManagedService>>>,
    /// Health check interval
    check_interval: Duration,
}

impl HealthManager {
    /// Create a new health manager
    pub fn new(service_manager: &ServiceManager) -> Self {
        Self {
            services: service_manager.services_ref(),
            check_interval: DEFAULT_CHECK_INTERVAL,
        }
    }

    /// Set the health check interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.check_interval = interval;
        self
    }

    /// Run the health check loop
    ///
    /// This should be spawned as a background task.
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

    /// Check health of all services
    async fn check_all(&self) {
        let services_to_check: Vec<(String, ServiceState, Option<u32>, bool, u32)> = {
            let services = self.services.read().await;
            services
                .iter()
                .filter(|(_, s)| s.state == ServiceState::Running)
                .map(|(name, s)| {
                    (
                        name.clone(),
                        s.state,
                        s.pid(),
                        s.config.restart_on_failure,
                        s.config.max_restarts,
                    )
                })
                .collect()
        };

        for (name, _state, pid, restart_on_failure, max_restarts) in services_to_check {
            if let Some(pid) = pid {
                if !self.is_process_running(pid) {
                    warn!("Service '{}' (PID {}) has died unexpectedly", name, pid);
                    self.handle_service_death(&name, restart_on_failure, max_restarts)
                        .await;
                } else {
                    debug!("Service '{}' (PID {}) is healthy", name, pid);
                }
            } else {
                // No PID means process didn't start properly
                warn!("Service '{}' has no PID, marking as failed", name);
                self.mark_failed(&name, "Process has no PID").await;
            }
        }
    }

    /// Handle a service that has died
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

                // Update state for restart
                service.state = ServiceState::Starting;
                service.restarts += 1;
                service.process = None;
                service.started_at = None;

                // Build and spawn command
                let config = service.config.clone();
                drop(services); // Release lock before spawning

                // Spawn the process
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

    /// Restart a service by spawning a new process
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

        let child = cmd.spawn()?;

        // Update service with new process
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

    /// Mark a service as failed
    async fn mark_failed(&self, name: &str, error: &str) {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            service.state = ServiceState::Failed;
            service.last_error = Some(error.to_string());
            service.process = None;
        }
    }

    /// Check if a process is running
    fn is_process_running(&self, pid: u32) -> bool {
        lib_daemon_core::is_process_running(pid)
    }
}

/// Health status summary
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Total number of services
    pub total: usize,
    /// Number of running services
    pub running: usize,
    /// Number of stopped services
    pub stopped: usize,
    /// Number of failed services
    pub failed: usize,
    /// Services that need attention (failed or restarting frequently)
    pub unhealthy: Vec<String>,
}

impl HealthStatus {
    /// Create a new health status from service map
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

    /// Check if all services are healthy
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
