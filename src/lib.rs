pub mod completions;
pub mod error;
pub mod plugin_registry;
pub mod plugin_runtime;
pub mod project_config;
pub mod self_update;

pub use error::{InstallerError, Result};
pub use plugin_registry::PluginManager;
pub use plugin_runtime::{PluginRuntime, RuntimeConfig};
