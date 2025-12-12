use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InstallConfig {
    pub install_path: Option<String>,
    pub options: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStatus {
    NotInstalled,
    Installed,
    UpdateAvailable,
}

/// Core trait for ADI components.
/// Implement this trait to add new installable components.
#[async_trait]
pub trait Component: Send + Sync {
    /// Returns component metadata
    fn info(&self) -> &ComponentInfo;

    /// Checks current installation status
    async fn status(&self) -> Result<InstallStatus>;

    /// Performs the installation
    async fn install(&self, config: &InstallConfig) -> Result<()>;

    /// Removes the component
    async fn uninstall(&self) -> Result<()>;

    /// Updates to latest version (default: reinstall)
    async fn update(&self, config: &InstallConfig) -> Result<()> {
        self.uninstall().await?;
        self.install(config).await
    }

    /// Validates installation prerequisites
    async fn validate_prerequisites(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}

/// Macro for easy component registration
#[macro_export]
macro_rules! register_components {
    ($($component:expr),* $(,)?) => {
        pub fn create_component_registry() -> $crate::registry::ComponentRegistry {
            let mut registry = $crate::registry::ComponentRegistry::new();
            $(
                registry.register(Box::new($component));
            )*
            registry
        }
    };
}
