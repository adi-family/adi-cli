use async_trait::async_trait;

use crate::component::{Component, ComponentInfo, InstallConfig, InstallStatus};
use crate::error::Result;
use crate::project_config::ProjectConfig;

use super::base_component::BaseIndexerComponent;

pub struct IndexerMcp {
    base: BaseIndexerComponent,
}

impl IndexerMcp {
    pub fn new() -> Self {
        let config = ProjectConfig::get();
        Self {
            base: BaseIndexerComponent::new("indexer-mcp", config.binaries.indexer_mcp.clone()),
        }
    }
}

impl Default for IndexerMcp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for IndexerMcp {
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
