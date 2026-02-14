use lib_i18n_core::fluent_bundle::FluentValue;
use lib_i18n_core::LocalizedError;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InstallerError {
    #[error("error-component-not-found")]
    ComponentNotFound(String),

    #[error("error-installation-failed")]
    InstallationFailed { component: String, reason: String },

    #[error("error-dependency-missing")]
    DependencyMissing {
        component: String,
        dependency: String,
    },

    #[error("error-config")]
    ConfigError(String),

    #[error("error-io")]
    Io(#[from] std::io::Error),

    #[error("error-serialization")]
    Serialization(#[from] serde_json::Error),

    #[error("error-already-installed")]
    AlreadyInstalled(String),

    #[error("error-uninstallation-failed")]
    UninstallationFailed { component: String, reason: String },

    #[error("error-registry")]
    Registry(#[from] lib_plugin_registry::RegistryError),

    #[error("error-plugin-not-found")]
    PluginNotFound { id: String },

    #[error("error-plugin-host")]
    PluginHost(#[from] lib_plugin_host::HostError),

    #[error("error-service")]
    Service(String),

    #[error("error-other")]
    Other(String),
}

impl LocalizedError for InstallerError {
    fn slug(&self) -> &str {
        match self {
            Self::ComponentNotFound(_) => "error-component-not-found",
            Self::InstallationFailed { .. } => "error-installation-failed",
            Self::DependencyMissing { .. } => "error-dependency-missing",
            Self::ConfigError(_) => "error-config",
            Self::Io(_) => "error-io",
            Self::Serialization(_) => "error-serialization",
            Self::AlreadyInstalled(_) => "error-already-installed",
            Self::UninstallationFailed { .. } => "error-uninstallation-failed",
            Self::Registry(_) => "error-registry",
            Self::PluginNotFound { .. } => "error-plugin-not-found",
            Self::PluginHost(_) => "error-plugin-host",
            Self::Service(_) => "error-service",
            Self::Other(_) => "error-other",
        }
    }

    fn i18n_args(&self) -> HashMap<String, FluentValue<'static>> {
        let mut args = HashMap::new();
        match self {
            Self::ComponentNotFound(name) => {
                args.insert("name".into(), FluentValue::from(name.clone()));
            }
            Self::InstallationFailed { component, reason } => {
                args.insert("component".into(), FluentValue::from(component.clone()));
                args.insert("reason".into(), FluentValue::from(reason.clone()));
            }
            Self::DependencyMissing {
                component,
                dependency,
            } => {
                args.insert("component".into(), FluentValue::from(component.clone()));
                args.insert("dependency".into(), FluentValue::from(dependency.clone()));
            }
            Self::ConfigError(detail) => {
                args.insert("detail".into(), FluentValue::from(detail.clone()));
            }
            Self::Io(e) => {
                args.insert("detail".into(), FluentValue::from(e.to_string()));
            }
            Self::Serialization(e) => {
                args.insert("detail".into(), FluentValue::from(e.to_string()));
            }
            Self::AlreadyInstalled(name) => {
                args.insert("name".into(), FluentValue::from(name.clone()));
            }
            Self::UninstallationFailed { component, reason } => {
                args.insert("component".into(), FluentValue::from(component.clone()));
                args.insert("reason".into(), FluentValue::from(reason.clone()));
            }
            Self::Registry(e) => {
                args.insert("detail".into(), FluentValue::from(e.to_string()));
            }
            Self::PluginNotFound { id } => {
                args.insert("id".into(), FluentValue::from(id.clone()));
            }
            Self::PluginHost(e) => {
                args.insert("detail".into(), FluentValue::from(e.to_string()));
            }
            Self::Service(detail) => {
                args.insert("detail".into(), FluentValue::from(detail.clone()));
            }
            Self::Other(detail) => {
                args.insert("detail".into(), FluentValue::from(detail.clone()));
            }
        }
        args
    }
}

impl From<lib_plugin_abi_v3::PluginError> for InstallerError {
    fn from(e: lib_plugin_abi_v3::PluginError) -> Self {
        InstallerError::Other(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, InstallerError>;
