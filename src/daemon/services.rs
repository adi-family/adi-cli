use super::log_buffer::LogBuffer;
use super::protocol::{ServiceConfig, ServiceInfo, ServiceState};
use crate::clienv;
use anyhow::Result;
use lib_daemon_core::is_process_running;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, ManagedService>>>,
    registry: ServiceRegistry,
    log_buffer: Arc<LogBuffer>,
}

pub struct ManagedService {
    pub config: ServiceConfig,
    pub state: ServiceState,
    pub process: Option<Child>,
    pub started_at: Option<Instant>,
    /// Number of restarts since daemon started
    pub restarts: u32,
    pub last_error: Option<String>,
}

impl ManagedService {
    pub fn new(config: ServiceConfig) -> Self {
        Self {
            config,
            state: ServiceState::Stopped,
            process: None,
            started_at: None,
            restarts: 0,
            last_error: None,
        }
    }

    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().and_then(|p| p.id())
    }

    pub fn uptime_secs(&self) -> Option<u64> {
        self.started_at.map(|t| t.elapsed().as_secs())
    }

    pub fn to_info(&self, name: &str) -> ServiceInfo {
        ServiceInfo {
            name: name.to_string(),
            state: self.state,
            pid: self.pid(),
            uptime_secs: self.uptime_secs(),
            restarts: self.restarts,
            last_error: self.last_error.clone(),
        }
    }
}

impl ServiceManager {
    pub fn new(log_buffer: Arc<LogBuffer>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            registry: ServiceRegistry::new(),
            log_buffer,
        }
    }

    pub fn log_buffer(&self) -> &Arc<LogBuffer> {
        &self.log_buffer
    }

    /// Discover daemon services from installed plugin manifests
    pub async fn discover_plugins(&mut self) -> Result<()> {
        self.registry.discover_plugins().await
    }

    /// Return service names that should be started automatically at daemon startup
    pub fn auto_start_names(&self) -> Vec<String> {
        self.registry.auto_start_names().to_vec()
    }

    pub async fn start(&self, name: &str, config: Option<ServiceConfig>) -> Result<()> {
        let mut services = self.services.write().await;

        let service = if let Some(s) = services.get_mut(name) {
            if s.state.is_running() {
                anyhow::bail!("Service '{}' is already running", name);
            }
            s
        } else {
            // Look up service config from registry
            let config = config
                .or_else(|| self.registry.get_config(name))
                .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", name))?;

            services.insert(name.to_string(), ManagedService::new(config));
            services.get_mut(name).unwrap()
        };

        service.state = ServiceState::Starting;
        service.last_error = None;

        let mut cmd = Command::new(&service.config.command);
        cmd.args(&service.config.args);

        for (key, value) in &service.config.env {
            cmd.env(key, value);
        }

        if let Some(ref dir) = service.config.working_dir {
            cmd.current_dir(std::path::Path::new(dir));
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                let pid = child.id();
                info!("Started service '{}' with PID {:?}", name, pid);

                spawn_log_readers(name, &mut child, &self.log_buffer);

                service.process = Some(child);
                service.state = ServiceState::Running;
                service.started_at = Some(Instant::now());

                Ok(())
            }
            Err(e) => {
                error!("Failed to start service '{}': {}", name, e);
                service.state = ServiceState::Failed;
                service.last_error = Some(e.to_string());
                Err(e.into())
            }
        }
    }

    pub async fn stop(&self, name: &str, force: bool) -> Result<()> {
        let mut services = self.services.write().await;

        let service = services
            .get_mut(name)
            .ok_or_else(|| anyhow::anyhow!("Unknown service: {}", name))?;

        if service.state.is_stopped() {
            return Ok(());
        }

        service.state = ServiceState::Stopping;

        if let Some(ref mut process) = service.process {
            if force {
                // SIGKILL
                info!("Force killing service '{}'", name);
                process.kill().await?;
            } else {
                // SIGTERM (graceful)
                info!("Stopping service '{}' gracefully", name);
                #[cfg(unix)]
                {
                    if let Some(pid) = process.id() {
                        unsafe {
                            libc::kill(pid as i32, libc::SIGTERM);
                        }
                    }
                }
                #[cfg(not(unix))]
                {
                    process.kill().await?;
                }

                // Wait for exit with timeout
                let timeout = tokio::time::Duration::from_secs(10);
                match tokio::time::timeout(timeout, process.wait()).await {
                    Ok(_) => {
                        debug!("Service '{}' stopped gracefully", name);
                    }
                    Err(_) => {
                        warn!("Service '{}' did not stop in time, force killing", name);
                        process.kill().await?;
                    }
                }
            }
        }

        service.state = ServiceState::Stopped;
        service.process = None;
        service.started_at = None;

        Ok(())
    }

    pub async fn restart(&self, name: &str) -> Result<()> {
        let config = {
            let services = self.services.read().await;
            services.get(name).map(|s| s.config.clone())
        };

        self.stop(name, false).await?;

        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(name) {
                service.restarts += 1;
            }
        }

        self.start(name, config).await
    }

    pub async fn list(&self) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, service)| service.to_info(name))
            .collect()
    }

    pub async fn get(&self, name: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(name).map(|s| s.to_info(name))
    }

    pub async fn stop_all(&self) {
        let names: Vec<String> = {
            let services = self.services.read().await;
            services.keys().cloned().collect()
        };

        for name in names {
            if let Err(e) = self.stop(&name, false).await {
                warn!("Failed to stop service '{}': {}", name, e);
            }
        }
    }

    pub async fn is_process_alive(&self, name: &str) -> bool {
        let services = self.services.read().await;
        if let Some(service) = services.get(name) {
            if let Some(pid) = service.pid() {
                return is_process_running(pid);
            }
        }
        false
    }

    pub async fn mark_failed(&self, name: &str, error: &str) {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            service.state = ServiceState::Failed;
            service.last_error = Some(error.to_string());
            service.process = None;
        }
    }

    pub async fn should_restart(&self, name: &str) -> bool {
        let services = self.services.read().await;
        if let Some(service) = services.get(name) {
            return service.config.restart_on_failure
                && service.restarts < service.config.max_restarts;
        }
        false
    }

    /// Get a clone of the services map for health checking
    pub fn services_ref(&self) -> Arc<RwLock<HashMap<String, ManagedService>>> {
        Arc::clone(&self.services)
    }
}

/// Spawn background tasks that read stdout/stderr from a child process into the LogBuffer.
fn spawn_log_readers(service_name: &str, child: &mut Child, log_buffer: &Arc<LogBuffer>) {
    if let Some(stdout) = child.stdout.take() {
        let buf = Arc::clone(log_buffer);
        let name = service_name.to_string();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                buf.push(&name, line);
            }
        });
    }
    if let Some(stderr) = child.stderr.take() {
        let buf = Arc::clone(log_buffer);
        let name = service_name.to_string();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                buf.push(&name, line);
            }
        });
    }
}

pub struct ServiceRegistry {
    builtin: HashMap<String, ServiceConfig>,
    auto_start: Vec<String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            builtin: HashMap::new(),
            auto_start: Vec::new(),
        }
    }

    pub fn get_config(&self, name: &str) -> Option<ServiceConfig> {
        self.builtin.get(name).cloned()
    }

    pub fn register(&mut self, name: String, config: ServiceConfig) {
        self.builtin.insert(name, config);
    }

    pub fn list(&self) -> Vec<String> {
        self.builtin.keys().cloned().collect()
    }

    pub fn auto_start_names(&self) -> &[String] {
        &self.auto_start
    }

    /// Scan installed plugin manifests for daemon service declarations
    pub async fn discover_plugins(&mut self) -> Result<()> {
        let plugins_dir = clienv::plugins_dir();
        if !plugins_dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&plugins_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = crate::plugin_runtime::find_plugin_toml_path(&path);
            let Some(manifest_path) = manifest_path else {
                continue;
            };

            match self.load_daemon_from_manifest(&manifest_path).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("Failed to load manifest {:?}: {}", manifest_path, e);
                }
            }
        }

        Ok(())
    }

    async fn load_daemon_from_manifest(&mut self, path: &std::path::Path) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let manifest: lib_plugin_manifest::PluginManifest = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse manifest: {}", e))?;

        let daemon_info = match &manifest.daemon {
            Some(info) => info,
            None => return Ok(()),
        };

        let plugin_id = &manifest.plugin.id;
        let exe = std::env::current_exe()
            .map_err(|e| anyhow::anyhow!("Failed to get exe path: {}", e))?;

        let config = ServiceConfig::new(exe.display().to_string())
            .args(["daemon", "run-service", plugin_id.as_str()])
            .env("RUST_LOG", "trace")
            .restart_on_failure(daemon_info.restart_on_failure)
            .max_restarts(daemon_info.max_restarts);

        if daemon_info.auto_start {
            info!("Discovered daemon service (auto-start): {}", plugin_id);
            self.auto_start.push(plugin_id.clone());
        } else {
            info!("Discovered daemon service: {}", plugin_id);
        }
        self.register(plugin_id.clone(), config);

        Ok(())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info() {
        let config = ServiceConfig::new("test");
        let service = ManagedService::new(config);

        assert_eq!(service.state, ServiceState::Stopped);
        assert!(service.pid().is_none());
        assert!(service.uptime_secs().is_none());
    }

    #[test]
    fn test_service_registry_empty() {
        let registry = ServiceRegistry::new();
        assert!(registry.get_config("hive").is_none());
        assert!(registry.get_config("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_service_manager_list() {
        let manager = ServiceManager::new(Arc::new(LogBuffer::default()));
        let list = manager.list().await;
        assert!(list.is_empty()); // No services started
    }
}
