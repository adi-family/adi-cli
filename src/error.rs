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
}

pub type Result<T> = std::result::Result<T, InstallerError>;
