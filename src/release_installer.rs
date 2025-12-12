use flate2::read::GzDecoder;
use serde::Deserialize;
use std::io::{Cursor, Read};
use std::path::Path;
use tar::Archive;
use zip::ZipArchive;

use crate::error::{InstallerError, Result};

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

pub struct ReleaseInstaller {
    repo_owner: String,
    repo_name: String,
    binary_name: String,
    tag_prefix: Option<String>,
}

impl ReleaseInstaller {
    pub fn new(repo_owner: &str, repo_name: &str, binary_name: &str) -> Self {
        Self {
            repo_owner: repo_owner.to_string(),
            repo_name: repo_name.to_string(),
            binary_name: binary_name.to_string(),
            tag_prefix: None,
        }
    }

    pub fn with_tag_prefix(mut self, tag_prefix: &str) -> Self {
        self.tag_prefix = Some(tag_prefix.to_string());
        self
    }

    pub async fn install_latest(&self, target_path: &Path) -> Result<String> {
        let release = self.fetch_latest_release().await?;
        let asset = self.select_asset(&release)?;

        self.download_and_extract(&asset.browser_download_url, &asset.name, target_path)
            .await?;

        Ok(release.tag_name)
    }

    async fn fetch_latest_release(&self) -> Result<GitHubRelease> {
        let client = reqwest::Client::builder()
            .user_agent("adi-installer")
            .build()
            .map_err(|e| InstallerError::InstallationFailed {
                component: self.repo_name.clone(),
                reason: format!("Failed to create HTTP client: {}", e),
            })?;

        if let Some(tag_prefix) = &self.tag_prefix {
            let url = format!(
                "https://api.github.com/repos/{}/{}/releases",
                self.repo_owner, self.repo_name
            );

            let response =
                client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| InstallerError::InstallationFailed {
                        component: self.repo_name.clone(),
                        reason: format!("Failed to fetch releases: {}", e),
                    })?;

            if !response.status().is_success() {
                return Err(InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("GitHub API returned status: {}", response.status()),
                });
            }

            let releases: Vec<GitHubRelease> =
                response
                    .json()
                    .await
                    .map_err(|e| InstallerError::InstallationFailed {
                        component: self.repo_name.clone(),
                        reason: format!("Failed to parse releases JSON: {}", e),
                    })?;

            releases
                .into_iter()
                .find(|release| release.tag_name.starts_with(tag_prefix))
                .ok_or_else(|| InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("No release found with tag prefix: {}", tag_prefix),
                })
        } else {
            let url = format!(
                "https://api.github.com/repos/{}/{}/releases/latest",
                self.repo_owner, self.repo_name
            );

            let response =
                client
                    .get(&url)
                    .send()
                    .await
                    .map_err(|e| InstallerError::InstallationFailed {
                        component: self.repo_name.clone(),
                        reason: format!("Failed to fetch release info: {}", e),
                    })?;

            if !response.status().is_success() {
                return Err(InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("GitHub API returned status: {}", response.status()),
                });
            }

            response
                .json()
                .await
                .map_err(|e| InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("Failed to parse release JSON: {}", e),
                })
        }
    }

    fn select_asset<'a>(&self, release: &'a GitHubRelease) -> Result<&'a GitHubAsset> {
        let platform = self.detect_platform();

        release
            .assets
            .iter()
            .find(|asset| {
                let name_lower = asset.name.to_lowercase();
                name_lower.contains(&platform.0) && name_lower.contains(&platform.1)
            })
            .ok_or_else(|| InstallerError::InstallationFailed {
                component: self.repo_name.clone(),
                reason: format!(
                    "No release asset found for platform: {}-{}",
                    platform.0, platform.1
                ),
            })
    }

    fn detect_platform(&self) -> (String, String) {
        let os = if cfg!(target_os = "macos") {
            "darwin"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else {
            "unknown"
        };

        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "unknown"
        };

        (os.to_string(), arch.to_string())
    }

    async fn download_and_extract(
        &self,
        url: &str,
        filename: &str,
        target_path: &Path,
    ) -> Result<()> {
        let client = reqwest::Client::new();
        let response =
            client
                .get(url)
                .send()
                .await
                .map_err(|e| InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("Failed to download asset: {}", e),
                })?;

        let bytes = response
            .bytes()
            .await
            .map_err(|e| InstallerError::InstallationFailed {
                component: self.repo_name.clone(),
                reason: format!("Failed to read response bytes: {}", e),
            })?;

        if filename.ends_with(".tar.gz") || filename.ends_with(".tgz") {
            self.extract_tar_gz(&bytes, target_path)?;
        } else if filename.ends_with(".zip") {
            self.extract_zip(&bytes, target_path)?;
        } else {
            // Direct binary download
            tokio::fs::write(target_path, &bytes).await?;
        }

        Ok(())
    }

    fn extract_tar_gz(&self, bytes: &[u8], target_path: &Path) -> Result<()> {
        let cursor = Cursor::new(bytes);
        let tar = GzDecoder::new(cursor);
        let mut archive = Archive::new(tar);

        for entry in archive
            .entries()
            .map_err(|e| InstallerError::InstallationFailed {
                component: self.repo_name.clone(),
                reason: format!("Failed to read tar archive: {}", e),
            })?
        {
            let mut entry = entry.map_err(|e| InstallerError::InstallationFailed {
                component: self.repo_name.clone(),
                reason: format!("Failed to read tar entry: {}", e),
            })?;

            let path = entry
                .path()
                .map_err(|e| InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("Failed to read entry path: {}", e),
                })?;

            if let Some(file_name) = path.file_name() {
                if file_name == self.binary_name.as_str() {
                    let mut buffer = Vec::new();
                    entry.read_to_end(&mut buffer).map_err(|e| {
                        InstallerError::InstallationFailed {
                            component: self.repo_name.clone(),
                            reason: format!("Failed to read binary: {}", e),
                        }
                    })?;

                    std::fs::write(target_path, buffer).map_err(|e| {
                        InstallerError::InstallationFailed {
                            component: self.repo_name.clone(),
                            reason: format!("Failed to write binary: {}", e),
                        }
                    })?;

                    return Ok(());
                }
            }
        }

        Err(InstallerError::InstallationFailed {
            component: self.repo_name.clone(),
            reason: format!("Binary '{}' not found in archive", self.binary_name),
        })
    }

    fn extract_zip(&self, bytes: &[u8], target_path: &Path) -> Result<()> {
        let cursor = Cursor::new(bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| InstallerError::InstallationFailed {
                component: self.repo_name.clone(),
                reason: format!("Failed to read zip archive: {}", e),
            })?;

        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| InstallerError::InstallationFailed {
                    component: self.repo_name.clone(),
                    reason: format!("Failed to read zip entry: {}", e),
                })?;

            if let Some(file_name) = Path::new(file.name()).file_name() {
                if file_name == self.binary_name.as_str() {
                    let mut buffer = Vec::new();
                    file.read_to_end(&mut buffer).map_err(|e| {
                        InstallerError::InstallationFailed {
                            component: self.repo_name.clone(),
                            reason: format!("Failed to read binary: {}", e),
                        }
                    })?;

                    std::fs::write(target_path, buffer).map_err(|e| {
                        InstallerError::InstallationFailed {
                            component: self.repo_name.clone(),
                            reason: format!("Failed to write binary: {}", e),
                        }
                    })?;

                    return Ok(());
                }
            }
        }

        Err(InstallerError::InstallationFailed {
            component: self.repo_name.clone(),
            reason: format!("Binary '{}' not found in archive", self.binary_name),
        })
    }
}
