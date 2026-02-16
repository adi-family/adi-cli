use anyhow::{anyhow, Result};
use lib_client_github::{no_auth, Client, Release, ReleaseAsset};
use lib_console_output::{out_info, out_success};
use lib_i18n_core::t;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

pub async fn check_for_updates() -> Result<Option<String>> {
    tracing::trace!(current = CURRENT_VERSION, "Checking for updates");
    let latest = fetch_latest_version().await?;
    tracing::trace!(current = CURRENT_VERSION, latest = %latest, "Version check complete");

    if version_is_newer(&latest, CURRENT_VERSION) {
        tracing::trace!(latest = %latest, "New version available");
        Ok(Some(latest))
    } else {
        tracing::trace!("Already at latest version");
        Ok(None)
    }
}

pub async fn self_update(force: bool) -> Result<()> {
    tracing::trace!(force = force, current = CURRENT_VERSION, "Starting self-update");
    out_info!("{}", t!("self-update-checking"));

    let latest_version = fetch_latest_version().await?;

    if !force && !version_is_newer(&latest_version, CURRENT_VERSION) {
        tracing::trace!("No update needed");
        out_success!("{}", t!("self-update-already-latest", "version" => CURRENT_VERSION));
        return Ok(());
    }

    out_info!("{}", t!("self-update-new-version", "current" => CURRENT_VERSION, "latest" => &latest_version));

    let current_exe = env::current_exe()?;
    let platform = detect_platform()?;
    tracing::trace!(platform = %platform, exe = %current_exe.display(), "Detected platform");

    out_info!("{}", t!("self-update-downloading"));
    let release = fetch_latest_release().await?;
    let asset = select_asset(&release, &platform)?;
    tracing::trace!(asset = %asset.name, url = %asset.browser_download_url, "Selected release asset");

    let temp_dir = env::temp_dir().join("adi-update");
    fs::create_dir_all(&temp_dir)?;

    let archive_path = temp_dir.join(&asset.name);
    tracing::trace!(dest = %archive_path.display(), "Downloading release archive");
    download_file(&asset.browser_download_url, &archive_path).await?;
    tracing::trace!("Download complete");

    out_info!("{}", t!("self-update-extracting"));
    let binary_path = extract_binary(&archive_path, &temp_dir)?;
    tracing::trace!(binary = %binary_path.display(), "Binary extracted");

    out_info!("{}", t!("self-update-installing"));
    tracing::trace!(src = %binary_path.display(), dest = %current_exe.display(), "Replacing binary");
    replace_binary(&binary_path, &current_exe)?;

    let _ = fs::remove_dir_all(&temp_dir);
    tracing::trace!("Temp directory cleaned up");

    out_success!("{}", t!("self-update-success", "version" => &latest_version));

    tracing::trace!(version = %latest_version, "Self-update complete");
    Ok(())
}

async fn fetch_latest_version() -> Result<String> {
    tracing::trace!("Fetching latest version from GitHub");
    let release = fetch_latest_release().await?;
    let version = release.tag_name.trim_start_matches("cli-v").to_string();
    tracing::trace!(tag = %release.tag_name, version = %version, "Parsed latest version");
    Ok(version)
}

fn build_github_client() -> Result<Client> {
    tracing::trace!("Building GitHub API client");
    Client::builder()
        .user_agent("adi-installer")
        .auth(no_auth())
        .build()
        .map_err(|e| anyhow!("Failed to build GitHub client: {}", e))
}

fn parse_repository() -> (&'static str, &'static str) {
    let url = REPOSITORY.trim_end_matches('/');
    let parts: Vec<&str> = url.rsplitn(3, '/').collect();
    (parts[1], parts[0])
}

async fn fetch_latest_release() -> Result<Release> {
    let (repo_owner, repo_name) = parse_repository();
    tracing::trace!(owner = %repo_owner, repo = %repo_name, "Fetching releases from GitHub");

    let client = build_github_client()?;
    let releases = client
        .list_releases(repo_owner, repo_name)
        .await
        .map_err(|e| anyhow!("Failed to fetch releases: {}", e))?;

    tracing::trace!(count = releases.len(), "Fetched releases");

    // Priority: cli-v* (new format), fallback to v* without component prefix (legacy)
    let cli_release = releases
        .iter()
        .find(|release| release.tag_name.starts_with("cli-v"))
        .or_else(|| {
            releases.iter().find(|release| {
                let tag = &release.tag_name;
                tag.starts_with('v') && !tag.contains("indexer-") && !tag.contains("cli-")
            })
        })
        .ok_or_else(|| anyhow!(t!("self-update-error-no-release")))?
        .clone();

    tracing::trace!(tag = %cli_release.tag_name, "Selected CLI release");
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
        return Err(anyhow!(t!("self-update-error-platform")));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        return Err(anyhow!(t!("self-update-error-arch")));
    };

    let platform = format!("{}-{}", arch, os);
    tracing::trace!(platform = %platform, "Detected platform");
    Ok(platform)
}

fn select_asset<'a>(release: &'a Release, platform: &str) -> Result<&'a ReleaseAsset> {
    tracing::trace!(platform = %platform, assets = release.assets.len(), "Selecting asset for platform");
    release
        .assets
        .iter()
        .find(|asset| asset.name.contains(platform))
        .ok_or_else(|| anyhow!(t!("self-update-error-no-asset", "platform" => platform)))
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    tracing::trace!(url = %url, dest = %dest.display(), "Downloading file");
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    tracing::trace!(bytes = bytes.len(), "Downloaded, writing to disk");
    fs::write(dest, bytes)?;
    Ok(())
}

fn extract_binary(archive_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
    let binary_name = if cfg!(windows) { "adi.exe" } else { "adi" };
    let binary_path = temp_dir.join(binary_name);
    tracing::trace!(archive = %archive_path.display(), binary_name = %binary_name, "Extracting binary from archive");

    if archive_path.extension().and_then(|s| s.to_str()) == Some("zip") {
        tracing::trace!("Using zip extraction");
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
                tracing::trace!("Binary extracted from zip");
                break;
            }
        }
    } else {
        tracing::trace!("Using tar.gz extraction");
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
                tracing::trace!("Binary extracted from tar.gz");
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
        tracing::trace!("Set binary permissions to 0o755");
    }

    Ok(binary_path)
}

fn replace_binary(new_binary: &Path, current_exe: &Path) -> Result<()> {
    tracing::trace!(src = %new_binary.display(), dest = %current_exe.display(), "Replacing binary");

    #[cfg(unix)]
    {
        fs::copy(new_binary, current_exe)?;
        tracing::trace!("Binary copied");

        // Re-sign: extracted binary loses its signature
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            tracing::trace!("Re-signing binary with ad-hoc signature (macOS)");
            let _ = Command::new("codesign")
                .args(["--remove-signature", current_exe.to_str().unwrap_or("")])
                .output();
            let _ = Command::new("codesign")
                .args(["-s", "-", current_exe.to_str().unwrap_or("")])
                .output();
            tracing::trace!("Binary re-signed");
        }

        Ok(())
    }

    #[cfg(windows)]
    {
        let old_exe = current_exe.with_extension("exe.old");
        tracing::trace!(old = %old_exe.display(), "Windows binary replacement");

        if old_exe.exists() {
            let _ = fs::remove_file(&old_exe);
        }

        fs::rename(current_exe, &old_exe)?;
        fs::copy(new_binary, current_exe)?;
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
