//! Plugin runtime for loading and managing plugins.
//!
//! Provides a unified interface for loading plugins, registering services,
//! and dispatching requests to plugin-provided HTTP/CLI handlers.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use lib_plugin_abi::{ServiceDescriptor, SERVICE_CLI_COMMANDS, SERVICE_HTTP_ROUTES};
use lib_plugin_host::{LoadedPluginV3, PluginConfig, PluginHost, PluginManagerV3, ServiceRegistry};
use lib_plugin_manifest::PluginManifest;

use crate::error::Result;

/// A discovered CLI command from a plugin manifest.
/// This is discovered by scanning plugin.toml files without loading binaries.
#[derive(Debug, Clone)]
pub struct PluginCliCommand {
    /// The command name (e.g., "tasks", "lint")
    pub command: String,
    /// The plugin ID that provides this command
    pub plugin_id: String,
    /// Human-readable description
    pub description: String,
    /// Optional short aliases
    pub aliases: Vec<String>,
}

/// Plugin runtime configuration.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Directory containing installed plugins.
    pub plugins_dir: PathBuf,
    /// Directory for plugin cache.
    pub cache_dir: PathBuf,
    /// Registry URL for downloading plugins.
    pub registry_url: Option<String>,
    /// Whether to require plugin signatures.
    pub require_signatures: bool,
    /// Host version for compatibility checking.
    pub host_version: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi");

        Self {
            plugins_dir: data_dir.join("plugins"),
            cache_dir: data_dir.join("cache"),
            registry_url: std::env::var("ADI_REGISTRY_URL").ok(),
            require_signatures: false,
            host_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Plugin runtime managing plugin lifecycle and service dispatch.
/// Uses RwLock because PluginHost requires mutable access for scanning/enabling.
pub struct PluginRuntime {
    host: Arc<RwLock<PluginHost>>,           // v2 loader
    manager_v3: Arc<RwLock<PluginManagerV3>>, // v3 loader
    config: RuntimeConfig,
}

impl PluginRuntime {
    /// Create a new plugin runtime with the given configuration.
    #[allow(clippy::arc_with_non_send_sync)]
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        // Ensure directories exist
        std::fs::create_dir_all(&config.plugins_dir)?;
        std::fs::create_dir_all(&config.cache_dir)?;

        let plugin_config = PluginConfig {
            plugins_dir: config.plugins_dir.clone(),
            cache_dir: config.cache_dir.clone(),
            registry_url: Some(
                config
                    .registry_url
                    .clone()
                    .unwrap_or_else(|| "https://adi-plugin-registry.the-ihor.com".to_string()),
            ),
            require_signatures: config.require_signatures,
            trusted_keys: Vec::new(),
            host_version: config.host_version.clone(),
        };

        let host = PluginHost::new(plugin_config)?;
        let manager_v3 = PluginManagerV3::new();

        Ok(Self {
            host: Arc::new(RwLock::new(host)),
            manager_v3: Arc::new(RwLock::new(manager_v3)),
            config,
        })
    }

    /// Create a runtime with default configuration.
    pub async fn with_defaults() -> Result<Self> {
        Self::new(RuntimeConfig::default()).await
    }

    /// Get the runtime configuration.
    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    /// Get the service registry.
    pub fn service_registry(&self) -> Arc<ServiceRegistry> {
        self.host.read().unwrap().service_registry().clone()
    }

    /// Scan and load all installed plugins.
    pub async fn load_all_plugins(&self) -> Result<()> {
        // Scan for v2 plugins
        let mut host = self.host.write().unwrap();
        host.scan_installed()?;

        // Enable all discovered v2 plugins
        let plugin_ids: Vec<String> = host.plugins().map(|p| p.id().to_string()).collect();
        drop(host); // Release lock before async operations

        for plugin_id in plugin_ids {
            if let Err(e) = self.load_plugin_internal(&plugin_id).await {
                tracing::warn!("Failed to enable plugin {}: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    /// Internal method to load a plugin by detecting its version
    async fn load_plugin_internal(&self, plugin_id: &str) -> Result<()> {
        // Find the plugin manifest
        let manifest = self.find_plugin_manifest(plugin_id)?;

        // Check API version
        match manifest.compatibility.api_version {
            3 => self.load_v3_plugin(&manifest).await?,
            _ => {
                // v2 or v1 - use old loader
                self.host.write().unwrap().enable(plugin_id)?;
            }
        }

        Ok(())
    }

    /// Load a v3 plugin
    async fn load_v3_plugin(&self, manifest: &PluginManifest) -> Result<()> {
        let plugin_dir = self.resolve_plugin_dir(&manifest.plugin.id)?;

        match LoadedPluginV3::load(manifest.clone(), &plugin_dir).await {
            Ok(loaded) => {
                let plugin_id = manifest.plugin.id.clone();

                // Register the plugin
                self.manager_v3.write().unwrap().register(loaded)?;

                tracing::info!("Loaded v3 plugin: {}", plugin_id);
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to load v3 plugin {}: {}", manifest.plugin.id, e);
                Err(crate::error::InstallerError::Other(format!(
                    "Failed to load v3 plugin: {}",
                    e
                )))
            }
        }
    }

    /// Find plugin manifest by ID
    fn find_plugin_manifest(&self, plugin_id: &str) -> Result<PluginManifest> {
        let plugin_dir = self.config.plugins_dir.join(plugin_id);

        if let Some(manifest_path) = Self::find_plugin_toml_path(&plugin_dir) {
            PluginManifest::from_file(&manifest_path)
                .map_err(|e| crate::error::InstallerError::Other(e.to_string()))
        } else {
            Err(crate::error::InstallerError::PluginNotFound {
                id: plugin_id.to_string(),
            })
        }
    }

    /// Resolve the plugin directory (handles versioned directories)
    fn resolve_plugin_dir(&self, plugin_id: &str) -> Result<PathBuf> {
        let plugin_dir = self.config.plugins_dir.join(plugin_id);

        // Check for .version file to get current version
        let version_file = plugin_dir.join(".version");
        if version_file.exists() {
            if let Ok(version) = std::fs::read_to_string(&version_file) {
                let version = version.trim();
                let versioned_dir = plugin_dir.join(version);
                if versioned_dir.exists() {
                    return Ok(versioned_dir);
                }
            }
        }

        // Fallback to plugin_dir itself
        Ok(plugin_dir)
    }

    /// Load a specific plugin by ID.
    pub async fn load_plugin(&self, plugin_id: &str) -> Result<()> {
        self.load_plugin_internal(plugin_id).await
    }

    /// Scan installed plugins and load a specific plugin by ID.
    /// This is useful when you only want to load one plugin without loading all.
    pub async fn scan_and_load_plugin(&self, plugin_id: &str) -> Result<()> {
        // Scan both v2 and v3 plugins
        self.host.write().unwrap().scan_installed()?;

        // Load the specific plugin (auto-detects version)
        self.load_plugin_internal(plugin_id).await
    }

    /// Unload a plugin.
    pub fn unload_plugin(&self, plugin_id: &str) -> Result<()> {
        self.host.write().unwrap().disable(plugin_id)?;
        Ok(())
    }

    /// List installed plugins.
    pub fn list_installed(&self) -> Vec<String> {
        self.host
            .read()
            .unwrap()
            .plugins()
            .map(|p| p.id().to_string())
            .collect()
    }

    /// List all registered services.
    pub fn list_services(&self) -> Vec<ServiceDescriptor> {
        self.service_registry().list()
    }

    /// Check if a service is available.
    pub fn has_service(&self, service_id: &str) -> bool {
        self.service_registry().has_service(service_id)
    }

    /// List plugins that provide HTTP routes.
    pub fn list_http_providers(&self) -> Vec<String> {
        self.service_registry()
            .list()
            .iter()
            .filter(|s| s.id.as_str() == SERVICE_HTTP_ROUTES)
            .map(|s| s.provider_id.as_str().to_string())
            .collect()
    }

    /// List plugins that provide CLI commands (legacy SERVICE_CLI_COMMANDS).
    pub fn list_cli_providers(&self) -> Vec<String> {
        self.service_registry()
            .list()
            .iter()
            .filter(|s| s.id.as_str() == SERVICE_CLI_COMMANDS)
            .map(|s| s.provider_id.as_str().to_string())
            .collect()
    }

    /// List plugins with runnable CLI interfaces.
    /// Returns (plugin_id, description) for each plugin with a `.cli` service.
    pub fn list_runnable_plugins(&self) -> Vec<(String, String)> {
        self.service_registry()
            .list()
            .iter()
            .filter(|s| s.id.as_str().ends_with(".cli"))
            .map(|s| {
                // Extract plugin_id from service_id (e.g., "adi.tasks.cli" -> "adi.tasks")
                let service_id = s.id.as_str();
                let plugin_id = service_id.strip_suffix(".cli").unwrap_or(service_id);
                (plugin_id.to_string(), s.description.as_str().to_string())
            })
            .collect()
    }

    /// Handle an HTTP request. Returns JSON response.
    pub fn handle_http_request(&self, handler_id: &str, request_json: &str) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_HTTP_ROUTES).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_HTTP_ROUTES.to_string(),
            }
        })?;

        let result = unsafe {
            handle.invoke(
                "handle_request",
                &format!(
                    r#"{{"handler_id":"{}","request":{}}}"#,
                    handler_id, request_json
                ),
            )?
        };
        Ok(result)
    }

    /// List HTTP routes. Returns JSON array of routes.
    pub fn list_http_routes(&self) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_HTTP_ROUTES).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_HTTP_ROUTES.to_string(),
            }
        })?;

        let result = unsafe { handle.invoke("list_routes", "{}")? };
        Ok(result)
    }

    /// Run a CLI command for a specific plugin. Returns result string.
    pub async fn run_cli_command(&self, plugin_id: &str, context_json: &str) -> Result<String> {
        // First, check if it's a v3 plugin
        {
            let manager = self.manager_v3.read().unwrap();
            if let Some(plugin) = manager.get_cli_commands(plugin_id) {
                // It's a v3 plugin - parse context and call async method
                let ctx = self.parse_cli_context(context_json)?;
                let result = plugin
                    .run_command(&ctx)
                    .await
                    .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;

                // Format result as JSON for compatibility
                return Ok(serde_json::to_string(&serde_json::json!({
                    "exit_code": result.exit_code,
                    "stdout": result.stdout,
                    "stderr": result.stderr,
                }))
                .unwrap());
            }
        }

        // Fall back to v2 plugin (JSON-RPC)
        let service_id = format!("{}.cli", plugin_id);
        let registry = self.service_registry();
        let handle = registry
            .lookup(&service_id)
            .ok_or_else(|| crate::error::InstallerError::PluginNotFound {
                id: service_id.clone(),
            })?;

        let result = unsafe { handle.invoke("run_command", context_json)? };
        Ok(result)
    }

    /// List CLI commands for a specific plugin. Returns JSON array of commands.
    pub async fn list_cli_commands(&self, plugin_id: &str) -> Result<String> {
        // First, check if it's a v3 plugin
        {
            let manager = self.manager_v3.read().unwrap();
            if let Some(plugin) = manager.get_cli_commands(plugin_id) {
                // It's a v3 plugin - call async method
                let commands = plugin.list_commands().await;
                return Ok(serde_json::to_string(&commands).unwrap());
            }
        }

        // Fall back to v2 plugin (JSON-RPC)
        let service_id = format!("{}.cli", plugin_id);
        let registry = self.service_registry();
        let handle = registry
            .lookup(&service_id)
            .ok_or_else(|| crate::error::InstallerError::PluginNotFound {
                id: service_id.clone(),
            })?;

        let result = unsafe { handle.invoke("list_commands", "{}")? };
        Ok(result)
    }

    /// Parse JSON context into CliContext for v3 plugins
    fn parse_cli_context(&self, context_json: &str) -> Result<lib_plugin_abi_v3::cli::CliContext> {
        use lib_plugin_abi_v3::cli::CliContext;
        use std::collections::HashMap;
        use std::path::PathBuf;

        // Parse the JSON context
        let value: serde_json::Value = serde_json::from_str(context_json)
            .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;

        // Extract fields
        let command = value
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let args: Vec<String> = value
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let cwd = value
            .get("cwd")
            .and_then(|v| v.as_str())
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Extract subcommand (first arg if present)
        let subcommand = args.first().cloned();

        // Build options map from remaining args
        let mut options = HashMap::new();
        if let Some(opts) = value.get("options").and_then(|v| v.as_object()) {
            for (k, v) in opts {
                options.insert(k.clone(), v.clone());
            }
        }

        // Get environment variables
        let env = std::env::vars().collect();

        Ok(CliContext {
            command,
            subcommand,
            args: args.into_iter().skip(1).collect(), // Skip subcommand from args
            options,
            cwd,
            env,
        })
    }

    /// Discover CLI commands from installed plugin manifests.
    ///
    /// This scans plugin.toml files in the plugins directory WITHOUT loading
    /// plugin binaries, making it fast for CLI command discovery.
    ///
    /// Returns a list of CLI commands with their plugin IDs, descriptions, and aliases.
    pub fn discover_cli_commands(&self) -> Vec<PluginCliCommand> {
        let mut commands = Vec::new();

        // Scan plugins directory for plugin.toml files
        let plugins_dir = &self.config.plugins_dir;
        if !plugins_dir.exists() {
            return commands;
        }

        // Each plugin is in a subdirectory: plugins/<plugin-id>/<version>/plugin.toml
        // or plugins/<plugin-id>/.version pointing to current version
        if let Ok(entries) = std::fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let plugin_dir = entry.path();
                if !plugin_dir.is_dir() {
                    continue;
                }

                // Try to find plugin.toml in versioned subdirectory
                let manifest_path = Self::find_plugin_toml_path(&plugin_dir);
                if let Some(manifest_path) = manifest_path {
                    if let Ok(manifest) = PluginManifest::from_file(&manifest_path) {
                        if let Some(cli) = &manifest.cli {
                            commands.push(PluginCliCommand {
                                command: cli.command.clone(),
                                plugin_id: manifest.plugin.id.clone(),
                                description: cli.description.clone(),
                                aliases: cli.aliases.clone(),
                            });
                        }
                    }
                }
            }
        }

        commands
    }

    /// Find the plugin.toml manifest in a plugin directory.
    /// Handles versioned directories (e.g., plugins/adi.tasks/0.8.8/plugin.toml)
    fn find_plugin_toml_path(plugin_dir: &PathBuf) -> Option<PathBuf> {
        // First, check for .version file to get current version
        let version_file = plugin_dir.join(".version");
        if version_file.exists() {
            if let Ok(version) = std::fs::read_to_string(&version_file) {
                let version = version.trim();
                let versioned_manifest = plugin_dir.join(version).join("plugin.toml");
                if versioned_manifest.exists() {
                    return Some(versioned_manifest);
                }
            }
        }

        // Fallback: check for plugin.toml directly in plugin dir
        let direct_manifest = plugin_dir.join("plugin.toml");
        if direct_manifest.exists() {
            return Some(direct_manifest);
        }

        // Fallback: scan subdirectories for plugin.toml
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            for entry in entries.flatten() {
                let subdir = entry.path();
                if subdir.is_dir() {
                    let manifest = subdir.join("plugin.toml");
                    if manifest.exists() {
                        return Some(manifest);
                    }
                }
            }
        }

        None
    }

    /// Find a plugin ID by command name or alias.
    /// Returns the plugin_id if found.
    pub fn find_plugin_by_command(&self, command: &str) -> Option<String> {
        let commands = self.discover_cli_commands();
        commands
            .iter()
            .find(|c| c.command == command || c.aliases.contains(&command.to_string()))
            .map(|c| c.plugin_id.clone())
    }
}

impl Clone for PluginRuntime {
    fn clone(&self) -> Self {
        Self {
            host: Arc::clone(&self.host),
            manager_v3: Arc::clone(&self.manager_v3),
            config: self.config.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let config = RuntimeConfig {
            plugins_dir: std::env::temp_dir().join("adi-test-plugins"),
            cache_dir: std::env::temp_dir().join("adi-test-cache"),
            registry_url: None,
            require_signatures: false,
            host_version: "0.1.0".to_string(),
        };

        let runtime = PluginRuntime::new(config).await;
        assert!(runtime.is_ok());
    }
}
