use async_trait::async_trait;

use crate::component::{Component, ComponentInfo, InstallConfig, InstallStatus};
use crate::error::Result;
use crate::project_config::ProjectConfig;

use super::base_tasks_component::BaseTasksComponent;

pub struct TasksCli {
    base: BaseTasksComponent,
}

impl TasksCli {
    pub fn new() -> Self {
        let config = ProjectConfig::get();
        Self {
            base: BaseTasksComponent::new("tasks-cli", config.binaries.tasks_cli.clone()),
        }
    }
}

impl Default for TasksCli {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for TasksCli {
    fn info(&self) -> &ComponentInfo {
        &self.base.info
    }

    async fn status(&self) -> Result<InstallStatus> {
        self.base.status().await
    }

    async fn install(&self, config: &InstallConfig) -> Result<()> {
        self.base.install(config).await
    }

    async fn uninstall(&self) -> Result<()> {
        self.base.uninstall().await
    }

    async fn validate_prerequisites(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }
}
