use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use lib_plugin_host::{LoadedPluginV3, PluginManagerV3};
use lib_plugin_manifest::PluginManifest;

use crate::error::Result;

/// Discovered from plugin.toml manifests without loading binaries.
#[derive(Debug, Clone)]
pub struct PluginCliCommand {
    pub command: String,
    pub plugin_id: String,
    pub description: String,
    pub aliases: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub plugins_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub registry_url: Option<String>,
    pub require_signatures: bool,
    pub host_version: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            plugins_dir: lib_plugin_host::PluginConfig::default_plugins_dir(),
            cache_dir: lib_plugin_host::PluginConfig::default_cache_dir(),
            registry_url: crate::clienv::registry_url_override(),
            require_signatures: false,
            host_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Uses RwLock because PluginManagerV3 requires mutable access for registration.
pub struct PluginRuntime {
    manager_v3: Arc<RwLock<PluginManagerV3>>,
    config: RuntimeConfig,
}

impl PluginRuntime {
    #[allow(clippy::arc_with_non_send_sync)]
    pub async fn new(config: RuntimeConfig) -> Result<Self> {
        tracing::trace!(plugins_dir = %config.plugins_dir.display(), cache_dir = %config.cache_dir.display(), "Creating plugin runtime");

        std::fs::create_dir_all(&config.plugins_dir)?;
        std::fs::create_dir_all(&config.cache_dir)?;

        let manager_v3 = PluginManagerV3::new();
        tracing::trace!("Plugin manager v3 initialized");

        Ok(Self {
            manager_v3: Arc::new(RwLock::new(manager_v3)),
            config,
        })
    }

    pub async fn with_defaults() -> Result<Self> {
        Self::new(RuntimeConfig::default()).await
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub async fn load_all_plugins(&self) -> Result<()> {
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
                    if entry.file_name() == lib_plugin_host::command_index::COMMANDS_DIR_NAME {
                        continue;
                    }
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

    async fn load_plugin_internal(&self, plugin_id: &str) -> Result<()> {
        tracing::trace!(plugin_id = %plugin_id, "Finding plugin manifest");

        let manifest = self.find_plugin_manifest(plugin_id)?;

        tracing::trace!(plugin_id = %plugin_id, version = %manifest.plugin.version, "Manifest found, loading v3 plugin");

        self.load_v3_plugin(&manifest).await
    }

    async fn load_v3_plugin(&self, manifest: &PluginManifest) -> Result<()> {
        let plugin_dir = self.resolve_plugin_dir(&manifest.plugin.id)?;
        tracing::trace!(plugin_id = %manifest.plugin.id, dir = %plugin_dir.display(), "Loading v3 plugin binary");

        match LoadedPluginV3::load(manifest.clone(), &plugin_dir).await {
            Ok(loaded) => {
                let plugin_id = manifest.plugin.id.clone();

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

    fn resolve_plugin_dir(&self, plugin_id: &str) -> Result<PathBuf> {
        let plugin_dir = self.config.plugins_dir.join(plugin_id);

        let latest_link = plugin_dir.join(lib_plugin_host::command_index::LATEST_LINK_NAME);
        if latest_link.is_symlink() {
            if let Ok(resolved) = std::fs::canonicalize(&latest_link) {
                tracing::trace!(plugin_id = %plugin_id, dir = %resolved.display(), "Resolved via latest symlink");
                return Ok(resolved);
            }
        }

        let version_file = plugin_dir.join(".version");
        if version_file.exists() {
            if let Ok(version) = std::fs::read_to_string(&version_file) {
                let version = version.trim();
                let versioned_dir = plugin_dir.join(version);
                if versioned_dir.exists() {
                    tracing::trace!(plugin_id = %plugin_id, version = %version, dir = %versioned_dir.display(), "Resolved via .version file");
                    return Ok(versioned_dir);
                }
            }
        }

        tracing::trace!(plugin_id = %plugin_id, dir = %plugin_dir.display(), "Using plugin directory directly");
        Ok(plugin_dir)
    }

    pub async fn scan_and_load_plugin(&self, plugin_id: &str) -> Result<()> {
        tracing::trace!(plugin_id = %plugin_id, "Scan-and-load single plugin");
        self.load_plugin_internal(plugin_id).await
    }

    pub fn list_installed(&self) -> Vec<String> {
        self.manager_v3
            .read()
            .unwrap()
            .list_plugins()
            .into_iter()
            .map(|p| p.id)
            .collect()
    }

    pub fn list_runnable_plugins(&self) -> Vec<(String, String)> {
        let manager = self.manager_v3.read().unwrap();
        manager
            .all_cli_commands()
            .into_iter()
            .map(|(id, _)| {
                let description = manager
                    .get_plugin(&id)
                    .and_then(|p| p.metadata().description)
                    .unwrap_or_default();
                (id, description)
            })
            .collect()
    }

    pub fn get_log_provider(&self, plugin_id: &str) -> Option<std::sync::Arc<dyn lib_plugin_abi_v3::logs::LogProvider>> {
        self.manager_v3.read().unwrap().get_log_provider(plugin_id)
    }

    pub fn get_daemon_service(&self, plugin_id: &str) -> Option<std::sync::Arc<dyn lib_plugin_abi_v3::daemon::DaemonService>> {
        self.manager_v3.read().unwrap().get_daemon_service(plugin_id)
    }

    pub async fn run_cli_command(&self, plugin_id: &str, context_json: &str) -> Result<String> {
        tracing::trace!(plugin_id = %plugin_id, "Running CLI command");

        let plugin = {
            let manager = self.manager_v3.read().unwrap();
            manager
                .get_cli_commands(plugin_id)
                .ok_or_else(|| crate::error::InstallerError::PluginNotFound {
                    id: plugin_id.to_string(),
                })?
        };

        let ctx = self.parse_cli_context(context_json)?;
        tracing::trace!(plugin_id = %plugin_id, command = %ctx.command, subcommand = ?ctx.subcommand, args = ?ctx.args, "Dispatching command to plugin");

        let result = plugin
            .run_command(&ctx)
            .await
            .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;

        tracing::trace!(plugin_id = %plugin_id, exit_code = result.exit_code, "Plugin command completed");

        Ok(serde_json::to_string(&serde_json::json!({
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
        }))
        .unwrap())
    }

    pub async fn list_cli_commands(&self, plugin_id: &str) -> Result<String> {
        let plugin = {
            let manager = self.manager_v3.read().unwrap();
            manager
                .get_cli_commands(plugin_id)
                .ok_or_else(|| crate::error::InstallerError::PluginNotFound {
                    id: plugin_id.to_string(),
                })?
        };

        let commands = plugin.list_commands().await;
        Ok(serde_json::to_string(&commands).unwrap())
    }

    fn parse_cli_context(&self, context_json: &str) -> Result<lib_plugin_abi_v3::cli::CliContext> {
        use lib_plugin_abi_v3::cli::CliContext;

        let value: serde_json::Value = serde_json::from_str(context_json)
            .map_err(|e| crate::error::InstallerError::Other(e.to_string()))?;

        let command = Self::json_str(&value, "command").unwrap_or_default();
        let args = Self::parse_json_args(&value);
        let cwd = Self::json_str(&value, "cwd")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        let subcommand = args.first().cloned();
        let mut options = Self::parse_json_options(&value);
        let remaining_args: Vec<String> = args.into_iter().skip(1).collect();
        let positional_args = Self::split_args_and_flags(&remaining_args, &mut options);

        Ok(CliContext {
            command,
            subcommand,
            args: positional_args,
            options,
            cwd,
            env: std::env::vars().collect(),
        })
    }

    fn json_str(value: &serde_json::Value, key: &str) -> Option<String> {
        value.get(key).and_then(|v| v.as_str()).map(String::from)
    }

    fn parse_json_args(value: &serde_json::Value) -> Vec<String> {
        value
            .get("args")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn parse_json_options(value: &serde_json::Value) -> std::collections::HashMap<String, serde_json::Value> {
        value
            .get("options")
            .and_then(|v| v.as_object())
            .map(|opts| opts.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    fn split_args_and_flags(
        args: &[String],
        options: &mut std::collections::HashMap<String, serde_json::Value>,
    ) -> Vec<String> {
        let mut positional = Vec::new();
        let mut i = 0;
        while i < args.len() {
            let Some(key) = args[i].strip_prefix("--") else {
                positional.push(args[i].clone());
                i += 1;
                continue;
            };
            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                options.insert(key.to_string(), serde_json::Value::String(args[i + 1].clone()));
                i += 2;
            } else {
                options.insert(key.to_string(), serde_json::Value::Bool(true));
                i += 1;
            }
        }
        positional
    }

    pub fn discover_cli_commands(&self) -> Vec<PluginCliCommand> {
        tracing::trace!("Discovering CLI commands");

        let plugins_dir = &self.config.plugins_dir;
        if !plugins_dir.exists() {
            tracing::trace!(dir = %plugins_dir.display(), "Plugins directory does not exist");
            return Vec::new();
        }

        let cmds_dir = lib_plugin_host::command_index::commands_dir(plugins_dir);
        if cmds_dir.exists() {
            let indexed = lib_plugin_host::command_index::list_indexed_commands(plugins_dir);
            if !indexed.is_empty() {
                tracing::trace!(count = indexed.len(), "Using command index (fast path)");
                return Self::commands_from_index(indexed);
            }
        }

        tracing::trace!("Command index missing or empty, falling back to full scan");
        let commands = self.discover_cli_commands_full_scan();

        if let Err(e) = lib_plugin_host::command_index::rebuild_index(plugins_dir) {
            tracing::warn!(error = %e, "Failed to rebuild command index");
        }

        commands
    }

    fn commands_from_index(indexed: Vec<(String, PathBuf)>) -> Vec<PluginCliCommand> {
        let mut seen = std::collections::HashMap::<PathBuf, PluginCliCommand>::new();

        for (_cmd_name, manifest_path) in indexed {
            if seen.contains_key(&manifest_path) {
                continue;
            }

            if let Ok(manifest) = PluginManifest::from_file(&manifest_path) {
                if let Some(cli) = &manifest.cli {
                    seen.insert(
                        manifest_path,
                        PluginCliCommand {
                            command: cli.command.clone(),
                            plugin_id: manifest.plugin.id.clone(),
                            description: cli.description.clone(),
                            aliases: cli.aliases.clone(),
                        },
                    );
                }
            }
        }

        seen.into_values().collect()
    }

    fn discover_cli_commands_full_scan(&self) -> Vec<PluginCliCommand> {
        let mut commands = Vec::new();
        let plugins_dir = &self.config.plugins_dir;

        if let Ok(entries) = std::fs::read_dir(plugins_dir) {
            for entry in entries.flatten() {
                let plugin_dir = entry.path();
                if !plugin_dir.is_dir() {
                    continue;
                }

                if entry.file_name() == lib_plugin_host::command_index::COMMANDS_DIR_NAME {
                    continue;
                }

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

        tracing::trace!(count = commands.len(), "Full scan discovery complete");
        commands
    }

    fn find_plugin_toml_path(plugin_dir: &std::path::Path) -> Option<PathBuf> {
        find_plugin_toml_path(plugin_dir)
    }

    pub fn find_plugin_by_command(&self, command: &str) -> Option<String> {
        tracing::trace!(command = %command, "Looking up plugin by command name or alias");

        let plugins_dir = &self.config.plugins_dir;

        if let Some(manifest_path) =
            lib_plugin_host::command_index::resolve_command(plugins_dir, command)
        {
            if let Ok(manifest) = PluginManifest::from_file(&manifest_path) {
                if manifest.cli.is_some() {
                    tracing::trace!(command = %command, plugin_id = %manifest.plugin.id, "Found via command index");
                    return Some(manifest.plugin.id);
                }
            }
        }

        tracing::trace!(command = %command, "Command index miss, falling back to full scan");
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

pub(crate) fn find_plugin_toml_path(plugin_dir: &std::path::Path) -> Option<PathBuf> {
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

    let direct_manifest = plugin_dir.join("plugin.toml");
    if direct_manifest.exists() {
        tracing::trace!(path = %direct_manifest.display(), "Found direct plugin.toml");
        return Some(direct_manifest);
    }

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
