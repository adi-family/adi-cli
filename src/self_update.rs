use anyhow::{anyhow, Result};
use console::style;
use reqwest;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::project_config::ProjectConfig;

#[derive(Debug, Clone, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Clone, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn check_for_updates() -> Result<Option<String>> {
    let latest = fetch_latest_version().await?;

    if version_is_newer(&latest, CURRENT_VERSION) {
        Ok(Some(latest))
    } else {
        Ok(None)
    }
}

pub async fn self_update(force: bool) -> Result<()> {
    println!("{}", style("Checking for updates...").cyan());

    let latest_version = fetch_latest_version().await?;

    if !force && !version_is_newer(&latest_version, CURRENT_VERSION) {
        println!(
            "{} You are already on the latest version ({})",
            style("✓").green(),
            CURRENT_VERSION
        );
        return Ok(());
    }

    println!(
        "{} New version available: {} → {}",
        style("→").cyan(),
        CURRENT_VERSION,
        latest_version
    );

    let current_exe = env::current_exe()?;
    let platform = detect_platform()?;

    println!("{} Downloading update...", style("→").cyan());
    let release = fetch_latest_release().await?;
    let asset = select_asset(&release, &platform)?;

    let temp_dir = env::temp_dir().join("adi-update");
    fs::create_dir_all(&temp_dir)?;

    let archive_path = temp_dir.join(&asset.name);
    download_file(&asset.browser_download_url, &archive_path).await?;

    println!("{} Extracting update...", style("→").cyan());
    let binary_path = extract_binary(&archive_path, &temp_dir)?;

    println!("{} Installing update...", style("→").cyan());
    replace_binary(&binary_path, &current_exe)?;

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    println!(
        "{} Successfully updated to version {}",
        style("✓").green(),
        latest_version
    );

    Ok(())
}

async fn fetch_latest_version() -> Result<String> {
    let release = fetch_latest_release().await?;
    let version = release.tag_name.trim_start_matches("cli-v").to_string();
    Ok(version)
}

async fn fetch_latest_release() -> Result<GitHubRelease> {
    let config = ProjectConfig::get();
    let (repo_owner, repo_name) = config.parse_repository();

    // Fetch all releases to filter for CLI-specific ones
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        repo_owner, repo_name
    );

    let client = reqwest::Client::builder()
        .user_agent("adi-installer")
        .build()?;

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "Failed to fetch release info: HTTP {}",
            response.status()
        ));
    }

    let releases: Vec<GitHubRelease> = response.json().await?;

    // Filter for CLI manager releases only
    // Priority: cli-v* (new format), fallback to v* without component prefix (legacy)
    // Reject: indexer-v* or any other component-prefixed releases

    // First, try to find a release with cli-v* prefix (new format)
    let cli_release = releases
        .iter()
        .find(|release| release.tag_name.starts_with("cli-v"))
        .or_else(|| {
            // Fallback: find legacy v* releases (without component prefix)
            releases.iter().find(|release| {
                let tag = &release.tag_name;
                tag.starts_with('v') && !tag.contains("indexer-") && !tag.contains("cli-")
            })
        })
        .ok_or_else(|| anyhow!("No CLI manager release found"))?
        .clone();

    Ok(cli_release)
}

fn detect_platform() -> Result<String> {
    let os = if cfg!(target_os = "macos") {
        "apple-darwin"
    } else if cfg!(target_os = "linux") {
        "unknown-linux-gnu"
    } else if cfg!(target_os = "windows") {
        "pc-windows-msvc"
    } else {
        return Err(anyhow!("Unsupported operating system"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err(anyhow!("Unsupported architecture"));
    };

    Ok(format!("{}-{}", arch, os))
}

fn select_asset<'a>(release: &'a GitHubRelease, platform: &str) -> Result<&'a GitHubAsset> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.contains(platform))
        .ok_or_else(|| anyhow!("No release asset found for platform: {}", platform))
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    fs::write(dest, bytes)?;
    Ok(())
}

fn extract_binary(archive_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
    let binary_name = if cfg!(windows) { "adi.exe" } else { "adi" };
    let binary_path = temp_dir.join(binary_name);

    if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        // Windows zip extraction
        use std::io::Read;
        use zip::ZipArchive;

        let file = fs::File::open(archive_path)?;
        let mut archive = ZipArchive::new(file)?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name() == binary_name {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;
                fs::write(&binary_path, buffer)?;
                break;
            }
        }
    } else {
        // Unix tar.gz extraction
        use flate2::read::GzDecoder;
        use std::io::Read;
        use tar::Archive;

        let tar_gz = fs::File::open(archive_path)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        for entry in archive.entries()? {
            let mut entry = entry?;
            let path = entry.path()?;

            if path.file_name().and_then(|s| s.to_str()) == Some(binary_name) {
                let mut buffer = Vec::new();
                entry.read_to_end(&mut buffer)?;
                fs::write(&binary_path, buffer)?;
                break;
            }
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    Ok(binary_path)
}

fn replace_binary(new_binary: &PathBuf, current_exe: &PathBuf) -> Result<()> {
    #[cfg(unix)]
    {
        // On Unix, we can replace the running binary
        fs::copy(new_binary, current_exe)?;
        Ok(())
    }

    #[cfg(windows)]
    {
        // On Windows, we need to use a different approach
        // Move current exe to .old, copy new binary, schedule deletion
        let old_exe = current_exe.with_extension("exe.old");

        if old_exe.exists() {
            let _ = fs::remove_file(&old_exe);
        }

        fs::rename(current_exe, &old_exe)?;
        fs::copy(new_binary, current_exe)?;

        // Schedule deletion of old binary on next boot
        // This is Windows-specific and simplified
        let _ = fs::remove_file(&old_exe);

        Ok(())
    }
}

fn version_is_newer(latest: &str, current: &str) -> bool {
    let latest = latest.trim_start_matches('v');
    let current = current.trim_start_matches('v');

    let parse_version =
        |v: &str| -> Vec<u32> { v.split('.').filter_map(|s| s.parse().ok()).collect() };

    let latest_parts = parse_version(latest);
    let current_parts = parse_version(current);

    for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
        if l > c {
            return true;
        } else if l < c {
            return false;
        }
    }

    latest_parts.len() > current_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(version_is_newer("1.0.1", "1.0.0"));
        assert!(version_is_newer("1.1.0", "1.0.0"));
        assert!(version_is_newer("2.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.0"));
        assert!(!version_is_newer("1.0.0", "1.0.1"));
        assert!(version_is_newer("v1.0.1", "v1.0.0"));
    }
}
