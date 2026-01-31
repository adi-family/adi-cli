use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("Component '{0}' not found")]
    ComponentNotFound(String),

    #[error("Installation failed for '{component}': {reason}")]
    InstallationFailed { component: String, reason: String },

    #[error("Dependency '{dependency}' required by '{component}' is not installed")]
    DependencyMissing {
        component: String,
        dependency: String,
    },

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Component '{0}' is already installed")]
    AlreadyInstalled(String),

    #[error("Uninstallation failed for '{component}': {reason}")]
    UninstallationFailed { component: String, reason: String },

    #[error("Registry error: {0}")]
    Registry(#[from] lib_plugin_registry::RegistryError),

    #[error("Plugin not found: {id}")]
    PluginNotFound { id: String },

    #[error("Plugin host error: {0}")]
    PluginHost(#[from] lib_plugin_host::HostError),

    #[error("Service error: {0}")]
    Service(String),

    #[error("{0}")]
    Other(String),
}

impl From<lib_plugin_abi_v3::PluginError> for InstallerError {
    fn from(e: lib_plugin_abi_v3::PluginError) -> Self {
        InstallerError::Other(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, InstallerError>;
