use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    /// Preferred language (e.g., "en-US", "zh-CN", "uk-UA")
    pub language: Option<String>,
    /// Preferred theme (e.g., "indigo", "scarlet", "emerald")
    pub theme: Option<String>,
    /// Power user mode - enables advanced features and verbose output
    pub power_user: Option<bool>,
}

impl UserConfig {
    /// $ADI_CONFIG_DIR/config.toml or ~/.config/adi/config.toml
    pub fn config_path() -> Result<PathBuf> {
        Ok(crate::clienv::config_dir().join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        tracing::trace!(path = %path.display(), "Loading user config");

        if !path.exists() {
            tracing::trace!("Config file does not exist, using defaults");
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        tracing::trace!(language = ?config.language, theme = ?config.theme, power_user = ?config.power_user, "User config loaded");
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        tracing::trace!(path = %path.display(), "Saving user config");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        tracing::trace!("User config saved");
        Ok(())
    }

    pub fn is_first_run() -> Result<bool> {
        let path = Self::config_path()?;
        let first_run = !path.exists();
        tracing::trace!(first_run = first_run, "Checking first run status");
        Ok(first_run)
    }

    pub fn is_interactive() -> bool {
        std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
}
