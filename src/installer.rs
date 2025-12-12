use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

use crate::component::{InstallConfig, InstallStatus};
use crate::error::{InstallerError, Result};
use crate::registry::ComponentRegistry;

pub struct Installer {
    registry: ComponentRegistry,
}

impl Installer {
    pub fn new(registry: ComponentRegistry) -> Self {
        Self { registry }
    }

    pub fn registry(&self) -> &ComponentRegistry {
        &self.registry
    }

    pub async fn install(&self, component_name: &str, config: &InstallConfig) -> Result<()> {
        let component = self.registry.get(component_name)?;
        let info = component.info();

        // Check dependencies
        for dep in &info.dependencies {
            if !self.registry.contains(dep) {
                return Err(InstallerError::DependencyMissing {
                    component: component_name.to_string(),
                    dependency: dep.clone(),
                });
            }

            let dep_component = self.registry.get(dep)?;
            if dep_component.status().await? == InstallStatus::NotInstalled {
                return Err(InstallerError::DependencyMissing {
                    component: component_name.to_string(),
                    dependency: dep.clone(),
                });
            }
        }

        // Validate prerequisites
        let warnings = component.validate_prerequisites().await?;
        for warning in warnings {
            println!("  Warning: {}", warning);
        }

        // Create progress bar
        let pb = create_progress_bar(&format!("Installing {}", info.name));

        // Perform installation
        let result = component.install(config).await;

        pb.finish_with_message(match &result {
            Ok(_) => format!("{} installed successfully", info.name),
            Err(e) => format!("Failed: {}", e),
        });

        result
    }

    pub async fn uninstall(&self, component_name: &str) -> Result<()> {
        let component = self.registry.get(component_name)?;
        let info = component.info();

        let pb = create_progress_bar(&format!("Uninstalling {}", info.name));

        let result = component.uninstall().await;

        pb.finish_with_message(match &result {
            Ok(_) => format!("{} uninstalled successfully", info.name),
            Err(e) => format!("Failed: {}", e),
        });

        result
    }

    pub async fn update(&self, component_name: &str, config: &InstallConfig) -> Result<()> {
        let component = self.registry.get(component_name)?;
        let info = component.info();

        let pb = create_progress_bar(&format!("Updating {}", info.name));

        let result = component.update(config).await;

        pb.finish_with_message(match &result {
            Ok(_) => format!("{} updated successfully", info.name),
            Err(e) => format!("Failed: {}", e),
        });

        result
    }

    pub async fn status(&self, component_name: &str) -> Result<InstallStatus> {
        let component = self.registry.get(component_name)?;
        component.status().await
    }
}

fn create_progress_bar(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}
