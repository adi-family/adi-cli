//! Plugin runtime for loading and managing plugins.
//!
//! Provides a unified interface for loading plugins, registering services,
//! and dispatching requests to plugin-provided MCP/HTTP/CLI handlers.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use lib_plugin_abi::{
    ServiceDescriptor, SERVICE_CLI_COMMANDS, SERVICE_HTTP_ROUTES, SERVICE_MCP_RESOURCES,
    SERVICE_MCP_TOOLS,
};
use lib_plugin_host::{PluginConfig, PluginHost, ServiceRegistry};

use crate::error::Result;

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
    host: Arc<RwLock<PluginHost>>,
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

        Ok(Self {
            host: Arc::new(RwLock::new(host)),
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
        let mut host = self.host.write().unwrap();
        host.scan_installed()?;

        // Enable all discovered plugins
        let plugin_ids: Vec<String> = host.plugins().map(|p| p.id().to_string()).collect();
        for plugin_id in plugin_ids {
            if let Err(e) = host.enable(&plugin_id) {
                tracing::warn!("Failed to enable plugin {}: {}", plugin_id, e);
            }
        }

        Ok(())
    }

    /// Load a specific plugin by ID.
    pub async fn load_plugin(&self, plugin_id: &str) -> Result<()> {
        self.host.write().unwrap().enable(plugin_id)?;
        Ok(())
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

    /// List plugins that provide MCP tools.
    pub fn list_mcp_tool_providers(&self) -> Vec<String> {
        self.service_registry()
            .list()
            .iter()
            .filter(|s| s.id.as_str() == SERVICE_MCP_TOOLS)
            .map(|s| s.provider_id.as_str().to_string())
            .collect()
    }

    /// List plugins that provide MCP resources.
    pub fn list_mcp_resource_providers(&self) -> Vec<String> {
        self.service_registry()
            .list()
            .iter()
            .filter(|s| s.id.as_str() == SERVICE_MCP_RESOURCES)
            .map(|s| s.provider_id.as_str().to_string())
            .collect()
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

    /// List plugins that provide CLI commands.
    pub fn list_cli_providers(&self) -> Vec<String> {
        self.service_registry()
            .list()
            .iter()
            .filter(|s| s.id.as_str() == SERVICE_CLI_COMMANDS)
            .map(|s| s.provider_id.as_str().to_string())
            .collect()
    }

    /// Call an MCP tool. Returns JSON result string.
    pub fn call_mcp_tool(&self, tool_name: &str, args: &str) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_MCP_TOOLS).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_MCP_TOOLS.to_string(),
            }
        })?;

        let result = unsafe {
            handle.invoke(
                "call_tool",
                &format!(r#"{{"name":"{}","args":{}}}"#, tool_name, args),
            )?
        };
        Ok(result)
    }

    /// List MCP tools. Returns JSON array of tools.
    pub fn list_mcp_tools(&self) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_MCP_TOOLS).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_MCP_TOOLS.to_string(),
            }
        })?;

        let result = unsafe { handle.invoke("list_tools", "{}")? };
        Ok(result)
    }

    /// Read an MCP resource. Returns JSON resource content.
    pub fn read_mcp_resource(&self, uri: &str) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_MCP_RESOURCES).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_MCP_RESOURCES.to_string(),
            }
        })?;

        let result = unsafe { handle.invoke("read_resource", &format!(r#"{{"uri":"{}"}}"#, uri))? };
        Ok(result)
    }

    /// List MCP resources. Returns JSON array of resources.
    pub fn list_mcp_resources(&self) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_MCP_RESOURCES).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_MCP_RESOURCES.to_string(),
            }
        })?;

        let result = unsafe { handle.invoke("list_resources", "{}")? };
        Ok(result)
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

    /// Run a CLI command. Returns JSON result.
    pub fn run_cli_command(&self, context_json: &str) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_CLI_COMMANDS).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_CLI_COMMANDS.to_string(),
            }
        })?;

        let result = unsafe { handle.invoke("run_command", context_json)? };
        Ok(result)
    }

    /// List CLI commands. Returns JSON array of commands.
    pub fn list_cli_commands(&self) -> Result<String> {
        let registry = self.service_registry();
        let handle = registry.lookup(SERVICE_CLI_COMMANDS).ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound {
                id: SERVICE_CLI_COMMANDS.to_string(),
            }
        })?;

        let result = unsafe { handle.invoke("list_commands", "{}")? };
        Ok(result)
    }
}

impl Clone for PluginRuntime {
    fn clone(&self) -> Self {
        Self {
            host: Arc::clone(&self.host),
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
