pub mod error;
pub mod http_server;
pub mod mcp_server;
pub mod plugin_registry;
pub mod plugin_runtime;
pub mod project_config;
pub mod self_update;

pub use error::{InstallerError, Result};
pub use http_server::{HttpServer, HttpServerConfig};
pub use mcp_server::McpServer;
pub use plugin_registry::PluginManager;
pub use plugin_runtime::{PluginRuntime, RuntimeConfig};
