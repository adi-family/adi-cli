use std::collections::HashSet;
use std::path::PathBuf;

use indicatif::{ProgressBar, ProgressStyle};
use lib_console_output::theme;
use lib_i18n_core::t;
use lib_plugin_host::{is_glob_pattern, PluginInstaller, UpdateCheck};
use lib_plugin_registry::{PluginEntry, PluginInfo, SearchResults};

use crate::error::Result;

/// CLI plugin manager — thin UI wrapper over `PluginInstaller`.
///
/// Delegates all business logic to `PluginInstaller` from `lib-plugin-host`,
/// adding progress bars, i18n messages, and user prompts.
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

        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("registry-cache");

        let install_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("plugins");

        tracing::trace!(registry_url = %registry_url, cache_dir = %cache_dir.display(), install_dir = %install_dir.display(), "Creating PluginManager");

        let installer = PluginInstaller::new(&registry_url, install_dir, cache_dir);

        Self { installer }
    }

    pub fn with_registry_url(url: &str) -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("registry-cache");

        let install_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("plugins");

        tracing::trace!(registry_url = %url, "Creating PluginManager with custom registry URL");

        let installer = PluginInstaller::new(url, install_dir, cache_dir);

        Self { installer }
    }

    // -- Delegated registry operations --

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

    // -- Install with UI --

    pub async fn install_plugin(&self, id: &str, version: Option<&str>) -> Result<()> {
        let platform = lib_plugin_manifest::current_platform();
        tracing::trace!(id = %id, version = ?version, platform = %platform, "Installing plugin");

        // Fetch info for pre-download message and progress bar sizing
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

        println!(
            "{}",
            t!("plugin-install-downloading",
                "id" => id,
                "version" => &info.version,
                "platform" => &platform
            )
        );

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

        println!(
            "{}",
            t!("plugin-install-extracting", "path" => &result.path.display().to_string())
        );

        {
            let prefix = t!("common-success-prefix");
            let msg = t!("plugin-install-success", "id" => id, "version" => &result.version);
            println!("{} {}", theme::success(prefix), msg);
        }

        Ok(())
    }

    /// Install a plugin and all its dependencies with UI feedback.
    pub async fn install_with_dependencies(&self, id: &str, version: Option<&str>) -> Result<()> {
        tracing::trace!(id = %id, version = ?version, "Installing plugin with dependencies");
        let mut installing = HashSet::new();

        // Check if already installed — provide user feedback
        if let Some(current_version) = self.installer.is_installed(id) {
            let prefix = t!("common-info-prefix");
            let msg = t!("plugin-install-already-installed",
                "id" => id,
                "version" => &current_version
            );
            println!("{} {}", theme::brand(prefix), msg);
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

        // Install with UI (progress bar, messages)
        self.install_plugin(id, version).await?;

        // Check dependencies using the library
        let deps = self.installer.get_dependencies(id);
        tracing::trace!(id = %id, deps = ?deps, "Checking plugin dependencies");
        for dep in deps {
            if !installing.contains(&dep) {
                println!("{}", t!("plugin-install-dependency", "id" => &dep));
                Box::pin(self.install_recursive(&dep, None, installing)).await?;
            }
        }

        Ok(())
    }

    // -- Uninstall with UI --

    pub async fn uninstall_plugin(&self, id: &str) -> Result<()> {
        tracing::trace!(id = %id, "Uninstalling plugin");
        println!("{}", t!("plugin-uninstall-progress", "id" => id));

        self.installer.uninstall(id).await?;
        tracing::trace!(id = %id, "Plugin uninstalled successfully");

        {
            let prefix = t!("common-success-prefix");
            let msg = t!("plugin-uninstall-success", "id" => id);
            println!("{} {}", theme::success(prefix), msg);
        }

        Ok(())
    }

    // -- Update with UI --

    pub async fn update_plugin(&self, id: &str) -> Result<()> {
        tracing::trace!(id = %id, "Checking for plugin update");
        match self.installer.check_update(id).await? {
            UpdateCheck::AlreadyLatest { version } => {
                tracing::trace!(id = %id, version = %version, "Plugin is already at latest version");
                let prefix = t!("common-info-prefix");
                let msg = t!("plugin-update-already-latest", "id" => id, "version" => &version);
                println!("{} {}", theme::brand(prefix), msg);
            }
            UpdateCheck::Available { current, latest } => {
                tracing::trace!(id = %id, current = %current, latest = %latest, "Plugin update available");
                println!(
                    "{}",
                    t!("plugin-update-available",
                        "id" => id,
                        "current" => &current,
                        "latest" => &latest
                    )
                );

                // Install the new version with UI
                self.install_plugin(id, Some(&latest)).await?;
            }
        }

        Ok(())
    }

    // -- Glob pattern install with UI --

    /// Install all plugins matching a glob pattern (e.g., "adi.lang.*")
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

        println!(
            "{}",
            t!("plugin-install-pattern-searching", "pattern" => pattern)
        );

        let matching = self.installer.find_matching(pattern).await?;

        if matching.is_empty() {
            let prefix = t!("common-warning-prefix");
            let msg = t!("plugin-install-pattern-none", "pattern" => pattern);
            println!("{} {}", theme::warning(prefix), msg);
            return Ok(());
        }

        println!(
            "{}",
            t!("plugin-install-pattern-found", "count" => &matching.len().to_string())
        );
        println!();

        for plugin in &matching {
            println!(
                "  {} {} - {}",
                theme::brand_bold(&plugin.id),
                theme::muted(format!("v{}", plugin.latest_version)),
                plugin.description
            );
        }

        println!();
        println!(
            "{}",
            t!("plugin-install-pattern-installing", "count" => &matching.len().to_string())
        );
        println!();

        let mut installed = 0;
        let mut failed = Vec::new();

        for plugin in &matching {
            match self.install_with_dependencies(&plugin.id, version).await {
                Ok(_) => {
                    installed += 1;
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to install {}: {}",
                        theme::warning("Warning:"),
                        plugin.id,
                        e
                    );
                    failed.push(plugin.id.clone());
                }
            }
            println!();
        }

        {
            let prefix = t!("common-success-prefix");
            let msg = t!("plugin-install-pattern-success", "count" => &installed.to_string());
            println!("{} {}", theme::success(prefix), msg);
        }

        if !failed.is_empty() {
            println!();
            let prefix = t!("common-warning-prefix");
            let msg = t!("plugin-install-pattern-failed");
            println!("{} {}", theme::warning(prefix), msg);
            for id in failed {
                println!("  - {}", id);
            }
        }

        Ok(())
    }
}
