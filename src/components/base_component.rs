use std::path::PathBuf;

use crate::component::{ComponentInfo, InstallConfig, InstallStatus};
use crate::error::Result;
use crate::project_config::ProjectConfig;
use crate::release_installer::ReleaseInstaller;

pub struct BaseIndexerComponent {
    pub info: ComponentInfo,
    pub binary_name: String,
}

impl BaseIndexerComponent {
    pub fn new(component_name: &str, binary_name: String) -> Self {
        let config = ProjectConfig::get();
        let component_config = config
            .get_component(component_name)
            .unwrap_or_else(|| panic!("{} component not found in config.toml", component_name));

        Self {
            info: component_config.to_component_info(),
            binary_name,
        }
    }

    pub fn binary_path(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("bin")
            .join(&self.binary_name)
    }

    pub fn version_file(&self) -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join(&self.info.name)
            .join(".version")
    }

    pub async fn status(&self) -> Result<InstallStatus> {
        if self.binary_path().exists() {
            Ok(InstallStatus::Installed)
        } else {
            Ok(InstallStatus::NotInstalled)
        }
    }

    pub async fn install(&self, _config: &InstallConfig) -> Result<()> {
        let config = ProjectConfig::get();
        let (repo_owner, repo_name) = config.parse_repository();

        let bin_dir = self.binary_path().parent().unwrap().to_path_buf();
        let version_dir = self.version_file().parent().unwrap().to_path_buf();

        tokio::fs::create_dir_all(&bin_dir).await?;
        tokio::fs::create_dir_all(&version_dir).await?;

        let installer = ReleaseInstaller::new(repo_owner, repo_name, &self.binary_name)
            .with_tag_prefix("indexer-");
        let version = installer.install_latest(&self.binary_path()).await?;

        tokio::fs::write(&self.version_file(), version.as_bytes()).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = tokio::fs::metadata(self.binary_path()).await?.permissions();
            perms.set_mode(0o755);
            tokio::fs::set_permissions(self.binary_path(), perms).await?;
        }

        Ok(())
    }

    pub async fn uninstall(&self) -> Result<()> {
        let binary_path = self.binary_path();
        let version_file = self.version_file();
        let version_dir = version_file.parent().unwrap();

        if binary_path.exists() {
            tokio::fs::remove_file(&binary_path).await?;
        }

        if version_dir.exists() {
            tokio::fs::remove_dir_all(version_dir).await?;
        }

        Ok(())
    }
}
