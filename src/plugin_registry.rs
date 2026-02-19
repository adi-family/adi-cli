use std::collections::HashSet;
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};
use lib_console_output::{theme, out_info, out_success, out_warn};
use lib_i18n_core::t;
use lib_plugin_host::{is_glob_pattern, PluginConfig, PluginInstaller, UpdateCheck};
use lib_plugin_registry::{PluginEntry, PluginInfo, SearchResults};

use crate::error::Result;

pub struct PluginManager {
    installer: PluginInstaller,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let registry_url = crate::clienv::registry_url();
        let config = PluginConfig::default().with_registry(&registry_url);

        tracing::trace!(
            registry_url = %registry_url,
            plugins_dir = %config.plugins_dir.display(),
            cache_dir = %config.cache_dir.display(),
            "Creating PluginManager"
        );

        Self {
            installer: PluginInstaller::from_config(&config),
        }
    }

    pub fn with_registry_url(url: &str) -> Self {
        let config = PluginConfig::default().with_registry(url);

        tracing::trace!(registry_url = %url, "Creating PluginManager with custom registry URL");

        Self {
            installer: PluginInstaller::from_config(&config),
        }
    }

    pub async fn search(&self, query: &str) -> Result<SearchResults> {
        tracing::trace!(query = %query, "Searching plugin registry");
        let results = self.installer.search(query).await?;
        tracing::trace!(packages = results.packages.len(), plugins = results.plugins.len(), "Search complete");
        Ok(results)
    }

    pub async fn list_plugins(&self) -> Result<Vec<PluginEntry>> {
        tracing::trace!("Listing available plugins from registry");
        let plugins = self.installer.list_available().await?;
        tracing::trace!(count = plugins.len(), "Available plugins fetched");
        Ok(plugins)
    }

    pub async fn get_plugin_info(&self, id: &str) -> Result<Option<PluginInfo>> {
        tracing::trace!(id = %id, "Fetching plugin info from registry");
        let info = self.installer.get_plugin_info(id).await?;
        tracing::trace!(id = %id, found = info.is_some(), "Plugin info result");
        Ok(info)
    }

    pub async fn list_installed(&self) -> Result<Vec<(String, String)>> {
        tracing::trace!("Listing installed plugins");
        let installed = self.installer.list_installed().await?;
        tracing::trace!(count = installed.len(), "Installed plugins listed");
        Ok(installed)
    }

    pub fn is_installed(&self, id: &str) -> Option<String> {
        let result = self.installer.is_installed(id);
        tracing::trace!(id = %id, installed = ?result, "Checking if plugin is installed");
        result
    }

    pub fn plugin_path(&self, id: &str) -> PathBuf {
        let path = self.installer.plugin_path(id);
        tracing::trace!(id = %id, path = %path.display(), "Resolved plugin path");
        path
    }

    pub async fn install_plugin(&self, id: &str, version: Option<&str>) -> Result<()> {
        let platform = lib_plugin_manifest::current_platform();
        tracing::trace!(id = %id, version = ?version, platform = %platform, "Installing plugin");

        let info = self.installer.get_plugin_info(id).await?;
        let info = info.ok_or_else(|| {
            crate::error::InstallerError::PluginNotFound { id: id.to_string() }
        })?;

        let platform_build = info
            .platforms
            .iter()
            .find(|p| p.platform == platform)
            .ok_or_else(|| {
                crate::error::InstallerError::Other(format!(
                    "Plugin {} does not support platform {}",
                    id, platform
                ))
            })?;

        out_info!("{}", t!("plugin-install-downloading",
            "id" => id,
            "version" => &info.version,
            "platform" => &platform
        ));

        let pb = ProgressBar::new(platform_build.size_bytes);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        let result = self
            .installer
            .install(id, version, |done, total| {
                pb.set_length(total);
                pb.set_position(done);
            })
            .await?;

        pb.finish_with_message("downloaded");
        tracing::trace!(id = %id, version = %result.version, path = %result.path.display(), "Plugin downloaded and extracted");

        out_info!("{}", t!("plugin-install-extracting", "path" => &result.path.display().to_string()));
        out_success!("{}", t!("plugin-install-success", "id" => id, "version" => &result.version));

        Ok(())
    }

    pub async fn install_with_dependencies(&self, id: &str, version: Option<&str>) -> Result<()> {
        tracing::trace!(id = %id, version = ?version, "Installing plugin with dependencies");
        let mut installing = HashSet::new();

        if let Some(current_version) = self.installer.is_installed(id) {
            out_info!("{}", t!("plugin-install-already-installed",
                "id" => id,
                "version" => &current_version
            ));
            return Ok(());
        }

        self.install_recursive(id, version, &mut installing).await
    }

    async fn install_recursive(
        &self,
        id: &str,
        version: Option<&str>,
        installing: &mut HashSet<String>,
    ) -> Result<()> {
        if installing.contains(id) {
            tracing::trace!(id = %id, "Skipping already-in-progress plugin install");
            return Ok(());
        }
        installing.insert(id.to_string());

        if self.installer.is_installed(id).is_some() {
            tracing::trace!(id = %id, "Plugin already installed, skipping");
            return Ok(());
        }

        self.install_plugin(id, version).await?;

        let deps = self.installer.get_dependencies(id);
        tracing::trace!(id = %id, deps = ?deps, "Checking plugin dependencies");
        for dep in deps {
            if !installing.contains(&dep) {
                out_info!("{}", t!("plugin-install-dependency", "id" => &dep));
                Box::pin(self.install_recursive(&dep, None, installing)).await?;
            }
        }

        Ok(())
    }

    pub async fn uninstall_plugin(&self, id: &str) -> Result<()> {
        tracing::trace!(id = %id, "Uninstalling plugin");
        out_info!("{}", t!("plugin-uninstall-progress", "id" => id));

        self.installer.uninstall(id).await?;
        tracing::trace!(id = %id, "Plugin uninstalled successfully");

        out_success!("{}", t!("plugin-uninstall-success", "id" => id));

        Ok(())
    }

    pub async fn update_plugin(&self, id: &str) -> Result<()> {
        tracing::trace!(id = %id, "Checking for plugin update");
        match self.installer.check_update(id).await? {
            UpdateCheck::AlreadyLatest { version } => {
                tracing::trace!(id = %id, version = %version, "Plugin is already at latest version");
                out_info!("{}", t!("plugin-update-already-latest", "id" => id, "version" => &version));
            }
            UpdateCheck::Available { current, latest } => {
                tracing::trace!(id = %id, current = %current, latest = %latest, "Plugin update available");
                out_info!("{}", t!("plugin-update-available",
                    "id" => id,
                    "current" => &current,
                    "latest" => &latest
                ));

                self.install_plugin(id, Some(&latest)).await?;
            }
        }

        Ok(())
    }

    pub async fn install_plugins_matching(
        &self,
        pattern: &str,
        version: Option<&str>,
    ) -> Result<()> {
        if !is_glob_pattern(pattern) {
            tracing::trace!(id = %pattern, "Not a glob pattern, installing single plugin");
            return self.install_with_dependencies(pattern, version).await;
        }

        tracing::trace!(pattern = %pattern, "Installing plugins matching glob pattern");
        out_info!("{}", t!("plugin-install-pattern-searching", "pattern" => pattern));

        let matching = self.installer.find_matching(pattern).await?;

        if matching.is_empty() {
            out_warn!("{}", t!("plugin-install-pattern-none", "pattern" => pattern));
            return Ok(());
        }

        Self::display_matching_plugins(&matching);
        out_info!("{}", t!("plugin-install-pattern-installing", "count" => &matching.len().to_string()));

        let failed = self.install_batch(&matching, version).await;

        Self::report_batch_results(matching.len() - failed.len(), &failed);

        Ok(())
    }

    fn display_matching_plugins(plugins: &[lib_plugin_registry::PluginEntry]) {
        out_info!("{}", t!("plugin-install-pattern-found", "count" => &plugins.len().to_string()));
        for plugin in plugins {
            out_info!("  {} {} - {}",
                theme::brand_bold(&plugin.id),
                theme::muted(format!("v{}", plugin.latest_version)),
                plugin.description
            );
        }
    }

    async fn install_batch(
        &self,
        plugins: &[lib_plugin_registry::PluginEntry],
        version: Option<&str>,
    ) -> Vec<String> {
        let mut failed = Vec::new();
        for plugin in plugins {
            if let Err(e) = self.install_with_dependencies(&plugin.id, version).await {
                out_warn!("Failed to install {}: {}", plugin.id, e);
                failed.push(plugin.id.clone());
            }
        }
        failed
    }

    fn report_batch_results(installed: usize, failed: &[String]) {
        out_success!("{}", t!("plugin-install-pattern-success", "count" => &installed.to_string()));
        if !failed.is_empty() {
            out_warn!("{}", t!("plugin-install-pattern-failed"));
            for id in failed {
                out_warn!("  - {}", id);
            }
        }
    }
}
