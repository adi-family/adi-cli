pub mod component;
pub mod components;
pub mod error;
pub mod installer;
pub mod project_config;
pub mod registry;
pub mod release_installer;
pub mod self_update;

pub use component::{Component, ComponentInfo, InstallConfig, InstallStatus};
pub use error::{InstallerError, Result};
pub use installer::Installer;
pub use registry::ComponentRegistry;
pub use release_installer::ReleaseInstaller;
