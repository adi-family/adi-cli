//! Plugin runtime for loading and managing plugins.
//!
//! Provides a unified interface for loading plugins, registering services,
//! and dispatching requests to plugin-provided HTTP/CLI handlers.
//!
//! All plugins use the v3 ABI (lib_plugin_abi_v3).

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use lib_plugin_host::{LoadedPluginV3, PluginManagerV3};
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
            registry_url: crate::clienv::registry_url_override(),
            require_signatures: false,
            host_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Plugin runtime managing plugin lifecycle and service dispatch.
/// Uses RwLock because PluginManagerV3 requires mutable access for registration.
pub struct PluginRuntime {
    manager_v3: Arc<RwLock<PluginManagerV3>>,
    config: RuntimeConfig,
}

impl PluginRuntime {
    /// Create a new plugin runtime with the given configuration.
    #[allow(clippy::arc_with_non_send_sync)]
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        tracing::trace!(plugins_dir = %config.plugins_dir.display(), cache_dir = %config.cache_dir.display(), "Creating plugin runtime");

        // Ensure directories exist
        std::fs::create_dir_all(&config.plugins_dir)?;
        std::fs::create_dir_all(&config.cache_dir)?;

        let manager_v3 = PluginManagerV3::new();
        tracing::trace!("Plugin manager v3 initialized");

        Ok(Self {
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

    /// Scan and load all installed plugins.
    pub async fn load_all_plugins(&self) -> Result<()> {
        // Scan plugins directory for installed plugins
        let plugins_dir = &self.config.plugins_dir;
        if !plugins_dir.exists() {
            tracing::trace!(dir = %plugins_dir.display(), "Plugins directory does not exist, skipping load");
            return Ok(());
        }

        tracing::trace!(dir = %plugins_dir.display(), "Scanning plugins directory");

        let mut plugin_ids = Vec::new();
        if let Ok(entries) = std::fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let plugin_dir = entry.path();
                if plugin_dir.is_dir() {
                    if let Some(name) = plugin_dir.file_name() {
                        plugin_ids.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }

        tracing::trace!(count = plugin_ids.len(), "Discovered plugin directories");

        for plugin_id in plugin_ids {
            tracing::trace!(plugin_id = %plugin_id, "Loading plugin");
            if let Err(e) = self.load_plugin_internal(&plugin_id).await {
                tracing::warn!("Failed to enable plugin {}: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    /// Internal method to load a plugin
    async fn load_plugin_internal(&self, plugin_id: &str) -> Result<()> {
        tracing::trace!(plugin_id = %plugin_id, "Finding plugin manifest");

        // Find the plugin manifest
        let manifest = self.find_plugin_manifest(plugin_id)?;

        tracing::trace!(plugin_id = %plugin_id, version = %manifest.plugin.version, "Manifest found, loading v3 plugin");

        // All plugins use v3 ABI
        self.load_v3_plugin(&manifest).await
    }

    /// Load a v3 plugin
    async fn load_v3_plugin(&self, manifest: &PluginManifest) -> Result<()> {
        let plugin_dir = self.resolve_plugin_dir(&manifest.plugin.id)?;
        tracing::trace!(plugin_id = %manifest.plugin.id, dir = %plugin_dir.display(), "Loading v3 plugin binary");

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
        tracing::trace!(plugin_id = %plugin_id, dir = %plugin_dir.display(), "Searching for plugin manifest");

        if let Some(manifest_path) = Self::find_plugin_toml_path(&plugin_dir) {
            tracing::trace!(path = %manifest_path.display(), "Found plugin manifest");
            PluginManifest::from_file(&manifest_path)
                .map_err(|e| crate::error::InstallerError::Other(e.to_string()))
        } else {
            tracing::trace!(plugin_id = %plugin_id, "Plugin manifest not found");
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
                    tracing::trace!(plugin_id = %plugin_id, version = %version, dir = %versioned_dir.display(), "Resolved versioned plugin directory");
                    return Ok(versioned_dir);
                }
            }
        }

        // Fallback to plugin_dir itself
        tracing::trace!(plugin_id = %plugin_id, dir = %plugin_dir.display(), "Using plugin directory directly (no version file)");
        Ok(plugin_dir)
    }

    /// Load a specific plugin by ID.
    pub async fn load_plugin(&self, plugin_id: &str) -> Result<()> {
        tracing::trace!(plugin_id = %plugin_id, "Loading single plugin");
        self.load_plugin_internal(plugin_id).await
    }

    /// Scan installed plugins and load a specific plugin by ID.
    /// This is useful when you only want to load one plugin without loading all.
    pub async fn scan_and_load_plugin(&self, plugin_id: &str) -> Result<()> {
        tracing::trace!(plugin_id = %plugin_id, "Scan-and-load single plugin");
        // Load the specific plugin
        self.load_plugin_internal(plugin_id).await
    }

    /// List installed plugins.
    pub fn list_installed(&self) -> Vec<String> {
        self.manager_v3
            .read()
            .unwrap()
            .list_plugins()
            .into_iter()
            .map(|p| p.id)
            .collect()
    }

    /// List plugins with runnable CLI interfaces.
    /// Returns (plugin_id, description) for each plugin with CLI commands.
    pub fn list_runnable_plugins(&self) -> Vec<(String, String)> {
        self.manager_v3
            .read()
            .unwrap()
            .all_cli_commands()
            .into_iter()
            .map(|(id, _)| {
                // Get description from plugin metadata if available
                let description = self
                    .manager_v3
                    .read()
                    .unwrap()
                    .get_plugin(&id)
                    .and_then(|p| p.metadata().description)
                    .unwrap_or_default();
                (id, description)
            })
            .collect()
    }

    /// Get a log provider for a specific plugin.
    pub fn get_log_provider(&self, plugin_id: &str) -> Option<std::sync::Arc<dyn lib_plugin_abi_v3::logs::LogProvider>> {
        self.manager_v3.read().unwrap().get_log_provider(plugin_id)
    }

    /// Run a CLI command for a specific plugin. Returns result string.
    pub async fn run_cli_command(&self, plugin_id: &str, context_json: &str) -> Result<String> {
        tracing::trace!(plugin_id = %plugin_id, "Running CLI command");

        let manager = self.manager_v3.read().unwrap();
        let plugin = manager
            .get_cli_commands(plugin_id)
            .ok_or_else(|| crate::error::InstallerError::PluginNotFound {
                id: plugin_id.to_string(),
            })?;

        // Parse context and call async method
        let ctx = self.parse_cli_context(context_json)?;
        tracing::trace!(plugin_id = %plugin_id, command = %ctx.command, subcommand = ?ctx.subcommand, args = ?ctx.args, "Dispatching command to plugin");
        drop(manager); // Release lock before async call

        let result = plugin
            .run_command(&ctx)
            .await
            .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;

        tracing::trace!(plugin_id = %plugin_id, exit_code = result.exit_code, "Plugin command completed");

        // Format result as JSON for compatibility
        Ok(serde_json::to_string(&serde_json::json!({
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }))
        .unwrap())
    }

    /// List CLI commands for a specific plugin. Returns JSON array of commands.
    pub async fn list_cli_commands(&self, plugin_id: &str) -> Result<String> {
        let manager = self.manager_v3.read().unwrap();
        let plugin = manager
            .get_cli_commands(plugin_id)
            .ok_or_else(|| crate::error::InstallerError::PluginNotFound {
                id: plugin_id.to_string(),
            })?;
        drop(manager); // Release lock before async call

        let commands = plugin.list_commands().await;
        Ok(serde_json::to_string(&commands).unwrap())
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

        // Build options map: first from explicit "options" field, then parse --flags from args
        let mut options = HashMap::new();
        if let Some(opts) = value.get("options").and_then(|v| v.as_object()) {
            for (k, v) in opts {
                options.insert(k.clone(), v.clone());
            }
        }

        // Parse --flags and --key value from args (skip subcommand at index 0)
        let remaining_args: Vec<String> = args.into_iter().skip(1).collect();
        let mut positional_args = Vec::new();
        let mut i = 0;
        while i < remaining_args.len() {
            let arg = &remaining_args[i];
            if let Some(key) = arg.strip_prefix("--") {
                // Check if next arg is a value (not another flag)
                if i + 1 < remaining_args.len() && !remaining_args[i + 1].starts_with("--") {
                    options.insert(key.to_string(), serde_json::Value::String(remaining_args[i + 1].clone()));
                    i += 2;
                } else {
                    options.insert(key.to_string(), serde_json::Value::Bool(true));
                    i += 1;
                }
            } else {
                positional_args.push(arg.clone());
                i += 1;
            }
        }

        // Get environment variables
        let env = std::env::vars().collect();

        Ok(CliContext {
            command,
            subcommand,
            args: positional_args,
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
        tracing::trace!("Discovering CLI commands from plugin manifests");

        let mut commands = Vec::new();

        // Scan plugins directory for plugin.toml files
        let plugins_dir = &self.config.plugins_dir;
        if !plugins_dir.exists() {
            tracing::trace!(dir = %plugins_dir.display(), "Plugins directory does not exist, no commands to discover");
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
                            tracing::trace!(command = %cli.command, plugin_id = %manifest.plugin.id, aliases = ?cli.aliases, "Discovered CLI command");
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

        tracing::trace!(count = commands.len(), "CLI command discovery complete");
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
                    tracing::trace!(path = %versioned_manifest.display(), "Found versioned plugin.toml");
                    return Some(versioned_manifest);
                }
            }
        }

        // Fallback: check for plugin.toml directly in plugin dir
        let direct_manifest = plugin_dir.join("plugin.toml");
        if direct_manifest.exists() {
            tracing::trace!(path = %direct_manifest.display(), "Found direct plugin.toml");
            return Some(direct_manifest);
        }

        // Fallback: scan subdirectories for plugin.toml
        if let Ok(entries) = std::fs::read_dir(plugin_dir) {
            for entry in entries.flatten() {
                let subdir = entry.path();
                if subdir.is_dir() {
                    let manifest = subdir.join("plugin.toml");
                    if manifest.exists() {
                        tracing::trace!(path = %manifest.display(), "Found plugin.toml in subdirectory");
                        return Some(manifest);
                    }
                }
            }
        }

        tracing::trace!(dir = %plugin_dir.display(), "No plugin.toml found");
        None
    }

    /// Find a plugin ID by command name or alias.
    /// Returns the plugin_id if found.
    pub fn find_plugin_by_command(&self, command: &str) -> Option<String> {
        tracing::trace!(command = %command, "Looking up plugin by command name or alias");
        let commands = self.discover_cli_commands();
        let result = commands
            .iter()
            .find(|c| c.command == command || c.aliases.contains(&command.to_string()))
            .map(|c| c.plugin_id.clone());
        tracing::trace!(command = %command, found = ?result, "Plugin lookup result");
        result
    }
}

impl Clone for PluginRuntime {
    fn clone(&self) -> Self {
        Self {
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
