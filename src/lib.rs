pub mod clienv;
pub mod completions;
pub mod daemon;
pub mod error;
pub mod plugin_registry;
pub mod plugin_runtime;
pub mod self_update;
pub mod user_config;

pub use error::{InstallerError, Result};
pub use plugin_registry::PluginManager;
pub use plugin_runtime::{PluginRuntime, RuntimeConfig};
pub use user_config::UserConfig;
