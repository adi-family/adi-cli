use async_trait::async_trait;

use crate::component::{Component, ComponentInfo, InstallConfig, InstallStatus};
use crate::error::Result;
use crate::project_config::ProjectConfig;

use super::base_component::BaseIndexerComponent;

pub struct LlmCodeIndexerHttp {
    base: BaseIndexerComponent,
}

impl LlmCodeIndexerHttp {
    pub fn new() -> Self {
        let config = ProjectConfig::get();
        Self {
            base: BaseIndexerComponent::new("indexer-http", config.binaries.indexer_http.clone()),
        }
    }
}

impl Default for LlmCodeIndexerHttp {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Component for LlmCodeIndexerHttp {
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
