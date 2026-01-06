use std::collections::HashSet;
use std::path::PathBuf;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use lib_i18n_core::t;
use lib_plugin_registry::{PluginEntry, RegistryClient, SearchKind, SearchResults};

use crate::error::Result;

const DEFAULT_REGISTRY_URL: &str = "https://adi-plugin-registry.the-ihor.com";

pub struct PluginManager {
    client: RegistryClient,
    install_dir: PathBuf,
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let registry_url =
            std::env::var("ADI_REGISTRY_URL").unwrap_or_else(|_| DEFAULT_REGISTRY_URL.to_string());

        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("registry-cache");

        let install_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("adi")
            .join("plugins");

        let client = RegistryClient::new(&registry_url).with_cache(cache_dir);

        Self {
            client,
            install_dir,
        }
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

        let client = RegistryClient::new(url).with_cache(cache_dir);

        Self {
            client,
            install_dir,
        }
    }

    pub async fn search(&self, query: &str) -> Result<SearchResults> {
        let results = self.client.search(query, SearchKind::All).await?;
        Ok(results)
    }

    pub async fn list_plugins(&self) -> Result<Vec<PluginEntry>> {
        let plugins = self.client.list_plugins().await?;
        Ok(plugins)
    }

    pub async fn install_plugin(&self, id: &str, version: Option<&str>) -> Result<()> {
        let platform = get_current_platform();

        // Get plugin info
        let info = if let Some(v) = version {
            self.client.get_plugin_version(id, v).await?
        } else {
            self.client.get_plugin_latest(id).await?
        };

        // Check if platform is supported
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

        // Create progress bar
        let pb = ProgressBar::new(platform_build.size_bytes);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        // Download plugin
        let bytes = self
            .client
            .download_plugin(id, &info.version, &platform, |done, total| {
                pb.set_length(total);
                pb.set_position(done);
            })
            .await?;

        pb.finish_with_message("downloaded");

        // Extract to install directory
        let plugin_dir = self.install_dir.join(id).join(&info.version);
        tokio::fs::create_dir_all(&plugin_dir).await?;

        println!(
            "{}",
            t!("plugin-install-extracting", "path" => &plugin_dir.display().to_string())
        );

        // Extract tarball
        let decoder = flate2::read::GzDecoder::new(&bytes[..]);
        let mut archive = tar::Archive::new(decoder);
        archive.unpack(&plugin_dir)?;

        // Write version file
        let version_file = self.install_dir.join(id).join(".version");
        tokio::fs::write(&version_file, info.version.as_bytes()).await?;

        // Set executable permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut entries = tokio::fs::read_dir(&plugin_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_file() {
                    if let Ok(metadata) = tokio::fs::metadata(&path).await {
                        let mut perms = metadata.permissions();
                        // Make files executable if they look like binaries
                        if !path
                            .extension()
                            .is_some_and(|e| e == "json" || e == "toml" || e == "txt" || e == "md")
                        {
                            perms.set_mode(0o755);
                            let _ = tokio::fs::set_permissions(&path, perms).await;
                        }
                    }
                }
            }
        }

        println!(
            "{} {}",
            style(t!("common-success-prefix")).green().bold(),
            t!("plugin-install-success", "id" => id, "version" => &info.version)
        );

        Ok(())
    }

    /// Install a plugin and all its dependencies.
    pub async fn install_with_dependencies(&self, id: &str, version: Option<&str>) -> Result<()> {
        // Track what we're installing to avoid cycles
        let mut installing = HashSet::new();

        // Check if already installed first to provide user feedback
        let version_file = self.install_dir.join(id).join(".version");
        if version_file.exists() {
            let current_version = tokio::fs::read_to_string(&version_file).await?;
            println!(
                "{} {}",
                style(t!("common-info-prefix")).cyan(),
                t!("plugin-install-already-installed",
                    "id" => id,
                    "version" => current_version.trim()
                )
            );
            return Ok(());
        }

        self.install_recursive(id, version, &mut installing).await
    }

    /// Recursively install a plugin and its dependencies.
    async fn install_recursive(
        &self,
        id: &str,
        version: Option<&str>,
        installing: &mut HashSet<String>,
    ) -> Result<()> {
        // Check for cycles
        if installing.contains(id) {
            return Ok(());
        }
        installing.insert(id.to_string());

        // Check if already installed
        let version_file = self.install_dir.join(id).join(".version");
        if version_file.exists() {
            // Already installed, skip
            return Ok(());
        }

        // Install the plugin first (to get the manifest)
        self.install_plugin(id, version).await?;

        // Now check for dependencies in the installed manifest
        let deps = self.get_plugin_dependencies(id).await;

        for dep in deps {
            if !installing.contains(&dep) {
                println!(
                    "{}",
                    t!("plugin-install-dependency", "id" => &dep)
                );
                // Recursively install dependency
                Box::pin(self.install_recursive(&dep, None, installing)).await?;
            }
        }

        Ok(())
    }

    /// Read dependencies from an installed plugin's manifest.
    async fn get_plugin_dependencies(&self, id: &str) -> Vec<String> {
        let mut deps = Vec::new();

        // Find the latest version directory
        let plugin_dir = self.install_dir.join(id);
        let version_file = plugin_dir.join(".version");

        let version = match tokio::fs::read_to_string(&version_file).await {
            Ok(v) => v.trim().to_string(),
            Err(_) => return deps,
        };

        let manifest_path = plugin_dir.join(&version).join("plugin.toml");

        let content = match tokio::fs::read_to_string(&manifest_path).await {
            Ok(c) => c,
            Err(_) => return deps,
        };

        // Parse TOML to extract depends_on
        if let Ok(table) = content.parse::<toml::Table>() {
            if let Some(compat) = table.get("compatibility").and_then(|c| c.as_table()) {
                if let Some(depends) = compat.get("depends_on").and_then(|d| d.as_array()) {
                    for dep in depends {
                        if let Some(s) = dep.as_str() {
                            deps.push(s.to_string());
                        }
                    }
                }
            }
        }

        deps
    }

    pub async fn uninstall_plugin(&self, id: &str) -> Result<()> {
        let plugin_dir = self.install_dir.join(id);

        if !plugin_dir.exists() {
            return Err(crate::error::InstallerError::Other(format!(
                "Plugin {} is not installed",
                id
            )));
        }

        println!("{}", t!("plugin-uninstall-progress", "id" => id));

        tokio::fs::remove_dir_all(&plugin_dir).await?;

        println!(
            "{} {}",
            style(t!("common-success-prefix")).green().bold(),
            t!("plugin-uninstall-success", "id" => id)
        );

        Ok(())
    }

    pub async fn update_plugin(&self, id: &str) -> Result<()> {
        let version_file = self.install_dir.join(id).join(".version");

        if !version_file.exists() {
            return Err(crate::error::InstallerError::Other(format!(
                "Plugin {} is not installed",
                id
            )));
        }

        let current_version = tokio::fs::read_to_string(&version_file).await?;
        let latest = self.client.get_plugin_latest(id).await?;

        if current_version.trim() == latest.version {
            println!(
                "{} {}",
                style(t!("common-info-prefix")).cyan(),
                t!("plugin-update-already-latest", "id" => id, "version" => &latest.version)
            );
            return Ok(());
        }

        println!(
            "{}",
            t!("plugin-update-available",
                "id" => id,
                "current" => current_version.trim(),
                "latest" => &latest.version
            )
        );

        // Remove old version directory but keep plugin root
        let old_version_dir = self.install_dir.join(id).join(current_version.trim());
        if old_version_dir.exists() {
            tokio::fs::remove_dir_all(&old_version_dir).await?;
        }

        // Install new version
        self.install_plugin(id, Some(&latest.version)).await
    }

    pub async fn list_installed(&self) -> Result<Vec<(String, String)>> {
        let mut installed = Vec::new();

        if !self.install_dir.exists() {
            return Ok(installed);
        }

        let mut entries = tokio::fs::read_dir(&self.install_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                let version_file = path.join(".version");
                if version_file.exists() {
                    let version = tokio::fs::read_to_string(&version_file).await?;
                    let name = path.file_name().unwrap().to_string_lossy().to_string();
                    installed.push((name, version.trim().to_string()));
                }
            }
        }

        Ok(installed)
    }

    pub fn plugin_path(&self, id: &str) -> PathBuf {
        self.install_dir.join(id)
    }

    /// Install all plugins matching a glob pattern (e.g., "adi.lang.*")
    pub async fn install_plugins_matching(
        &self,
        pattern: &str,
        version: Option<&str>,
    ) -> Result<()> {
        if !is_pattern(pattern) {
            // Not a pattern, install single plugin with dependencies
            return self.install_with_dependencies(pattern, version).await;
        }

        println!(
            "{}",
            t!("plugin-install-pattern-searching", "pattern" => pattern)
        );

        // Fetch all available plugins
        let all_plugins = self.list_plugins().await?;

        // Filter plugins matching the pattern
        let matching: Vec<_> = all_plugins
            .iter()
            .filter(|p| matches_pattern(&p.id, pattern))
            .collect();

        if matching.is_empty() {
            println!(
                "{} {}",
                style(t!("common-warning-prefix")).yellow(),
                t!("plugin-install-pattern-none", "pattern" => pattern)
            );
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
                style(&plugin.id).cyan().bold(),
                style(format!("v{}", plugin.latest_version)).dim(),
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
                        style("Warning:").yellow(),
                        plugin.id,
                        e
                    );
                    failed.push(plugin.id.clone());
                }
            }
            println!(); // Blank line between installs
        }

        println!(
            "{} {}",
            style(t!("common-success-prefix")).green().bold(),
            t!("plugin-install-pattern-success", "count" => &installed.to_string())
        );

        if !failed.is_empty() {
            println!();
            println!("{} {}", style(t!("common-warning-prefix")).yellow(), t!("plugin-install-pattern-failed"));
            for id in failed {
                println!("  - {}", id);
            }
        }

        Ok(())
    }
}

fn get_current_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let os_name = match os {
        "macos" => "darwin",
        "linux" => "linux",
        "windows" => "windows",
        other => other,
    };

    let arch_name = arch;

    format!("{}-{}", os_name, arch_name)
}

/// Check if a pattern contains wildcards
fn is_pattern(s: &str) -> bool {
    s.contains('*')
}

/// Match a string against a simple glob pattern (supports * wildcard)
fn matches_pattern(s: &str, pattern: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        // No wildcards, exact match
        return s == pattern;
    }

    let mut pos = 0;

    for (i, part) in parts.iter().enumerate() {
        if i == 0 {
            // First part must match at start
            if !s.starts_with(part) {
                return false;
            }
            pos = part.len();
        } else if i == parts.len() - 1 {
            // Last part must match at end
            if !s.ends_with(part) {
                return false;
            }
        } else {
            // Middle parts must exist in order
            if let Some(found_pos) = s[pos..].find(part) {
                pos += found_pos + part.len();
            } else {
                return false;
            }
        }
    }

    true
}
