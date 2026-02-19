use super::protocol::{ServiceConfig, ServiceInfo, ServiceState};
use crate::clienv;
use anyhow::Result;
use lib_daemon_core::is_process_running;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;
use tokio::process::{Child, Command};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Manages plugin services as child processes
pub struct ServiceManager {
    /// Running services by name
    services: Arc<RwLock<HashMap<String, ManagedService>>>,
    /// Service registry for discovering plugin services
    registry: ServiceRegistry,
}

/// A managed service with its process and metadata
pub struct ManagedService {
    /// Service configuration
    pub config: ServiceConfig,
    /// Current state
    pub state: ServiceState,
    /// Child process (if running)
    pub process: Option<Child>,
    /// Time when service was started
    pub started_at: Option<Instant>,
    /// Number of restarts since daemon started
    pub restarts: u32,
    /// Last error message
    pub last_error: Option<String>,
}

impl ManagedService {
    /// Create a new managed service
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

    /// Get current PID if running
    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().and_then(|p| p.id())
    }

    /// Get uptime in seconds if running
    pub fn uptime_secs(&self) -> Option<u64> {
        self.started_at.map(|t| t.elapsed().as_secs())
    }

    /// Convert to ServiceInfo for IPC responses
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
    /// Create a new service manager
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            registry: ServiceRegistry::new(),
        }
    }

    /// Start a service
    pub async fn start(&self, name: &str, config: Option<ServiceConfig>) -> Result<()> {
        let mut services = self.services.write().await;

        // Get or create service entry
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

        // Update state
        service.state = ServiceState::Starting;
        service.last_error = None;

        // Build command
        let mut cmd = Command::new(&service.config.command);
        cmd.args(&service.config.args);

        // Set environment
        for (key, value) in &service.config.env {
            cmd.env(key, value);
        }

        // Set working directory
        if let Some(ref dir) = service.config.working_dir {
            cmd.current_dir(std::path::Path::new(dir));
        }

        // Capture output for logging
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process
        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                info!("Started service '{}' with PID {:?}", name, pid);

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

    /// Stop a service
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

    /// Restart a service
    pub async fn restart(&self, name: &str) -> Result<()> {
        // Get config before stopping
        let config = {
            let services = self.services.read().await;
            services.get(name).map(|s| s.config.clone())
        };

        // Stop if running
        self.stop(name, false).await?;

        // Increment restart counter
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(name) {
                service.restarts += 1;
            }
        }

        // Start again
        self.start(name, config).await
    }

    /// List all services
    pub async fn list(&self) -> Vec<ServiceInfo> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, service)| service.to_info(name))
            .collect()
    }

    /// Get a specific service
    pub async fn get(&self, name: &str) -> Option<ServiceInfo> {
        let services = self.services.read().await;
        services.get(name).map(|s| s.to_info(name))
    }

    /// Stop all services
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

    /// Check if a service process is still running
    pub async fn is_process_alive(&self, name: &str) -> bool {
        let services = self.services.read().await;
        if let Some(service) = services.get(name) {
            if let Some(pid) = service.pid() {
                return is_process_running(pid);
            }
        }
        false
    }

    /// Mark a service as failed
    pub async fn mark_failed(&self, name: &str, error: &str) {
        let mut services = self.services.write().await;
        if let Some(service) = services.get_mut(name) {
            service.state = ServiceState::Failed;
            service.last_error = Some(error.to_string());
            service.process = None;
        }
    }

    /// Get service restart policy
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

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry for discovering plugin service configurations
pub struct ServiceRegistry {
    /// Built-in service configs
    builtin: HashMap<String, ServiceConfig>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        let mut builtin = HashMap::new();

        // Register built-in services
        // These will be overridden by plugin discovery

        // Hive service
        builtin.insert(
            "hive".to_string(),
            ServiceConfig::new("adi")
                .args(["run", "adi.hive", "serve"])
                .env("RUST_LOG", "info")
                .restart_on_failure(true)
                .max_restarts(3),
        );

        // Indexer service
        builtin.insert(
            "indexer".to_string(),
            ServiceConfig::new("adi")
                .args(["run", "adi.indexer", "serve"])
                .env("RUST_LOG", "info")
                .restart_on_failure(true)
                .max_restarts(3),
        );

        // LLM Proxy service
        builtin.insert(
            "llm-proxy".to_string(),
            ServiceConfig::new("adi")
                .args(["run", "adi.llm-proxy", "serve"])
                .env("RUST_LOG", "info")
                .restart_on_failure(true)
                .max_restarts(3),
        );

        Self { builtin }
    }

    /// Get service configuration by name
    pub fn get_config(&self, name: &str) -> Option<ServiceConfig> {
        self.builtin.get(name).cloned()
    }

    /// Register a plugin service
    pub fn register(&mut self, name: String, config: ServiceConfig) {
        self.builtin.insert(name, config);
    }

    /// List all registered services
    pub fn list(&self) -> Vec<String> {
        self.builtin.keys().cloned().collect()
    }

    /// Discover services from installed plugins
    ///
    /// Reads plugin manifests to find services with `[package.metadata.plugin.service]`
    pub async fn discover_plugins(&mut self) -> Result<()> {
        let plugins_dir = clienv::plugins_dir();

        if !plugins_dir.exists() {
            return Ok(());
        }

        // Scan plugin directories for manifests
        let mut entries = tokio::fs::read_dir(&plugins_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.toml");
                if manifest_path.exists() {
                    if let Err(e) = self.load_plugin_manifest(&manifest_path).await {
                        warn!("Failed to load plugin manifest {:?}: {}", manifest_path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Load a plugin manifest and register its service if defined
    async fn load_plugin_manifest(&self, path: &std::path::Path) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let manifest: toml::Value = toml::from_str(&content)?;

        // Check for service configuration
        if let Some(service) = manifest
            .get("package")
            .and_then(|p| p.get("metadata"))
            .and_then(|m| m.get("plugin"))
            .and_then(|p| p.get("service"))
        {
            let name = service
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Service missing name"))?;

            let command = service
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("serve");

            debug!("Discovered plugin service: {} (command: {})", name, command);

            // Service will be registered when the plugin is loaded
        }

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
    fn test_service_registry() {
        let registry = ServiceRegistry::new();
        assert!(registry.get_config("hive").is_some());
        assert!(registry.get_config("indexer").is_some());
        assert!(registry.get_config("nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_service_manager_list() {
        let manager = ServiceManager::new();
        let list = manager.list().await;
        assert!(list.is_empty()); // No services started
    }
}
