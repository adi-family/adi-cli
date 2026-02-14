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
}

impl UserConfig {
    /// Get path to user config file ($ADI_CONFIG_DIR/config.toml or ~/.config/adi/config.toml)
    pub fn config_path() -> Result<PathBuf> {
        Ok(crate::clienv::config_dir().join("config.toml"))
    }

    /// Load user config from disk, returns default if file doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {}", path.display()))?;

        let config: Self = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config from {}", path.display()))?;

        Ok(config)
    }

    /// Save user config to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let content = toml::to_string_pretty(self).context("Failed to serialize config to TOML")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write config to {}", path.display()))?;

        Ok(())
    }

    /// Check if this is the first run (config file doesn't exist)
    pub fn is_first_run() -> Result<bool> {
        let path = Self::config_path()?;
        Ok(!path.exists())
    }

    /// Check if we're in an interactive session (TTY)
    pub fn is_interactive() -> bool {
        std::io::IsTerminal::is_terminal(&std::io::stdin())
    }
}
