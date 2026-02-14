// Centralized environment variable access for the ADI CLI crate.
// All env var names are defined in the `EnvVar` enum (via `env_vars!` macro),
// and typed getter functions below provide safe access throughout `crates/cli`.
// Other crates should never read these env vars directly — use this module instead.

use std::path::PathBuf;

use lib_env_parse::{env_bool_default_true, env_opt, env_or, env_vars};

env_vars! {
    AdiConfigDir       => "ADI_CONFIG_DIR",
    AdiTheme           => "ADI_THEME",
    AdiLang            => "ADI_LANG",
    Lang               => "LANG",
    AdiAutoInstall     => "ADI_AUTO_INSTALL",
    AdiRegistryUrl     => "ADI_REGISTRY_URL",
    SignalingServerUrl  => "SIGNALING_SERVER_URL",
}

const FALLBACK_CONFIG_DIR: &str = "~/.config";
const ADI_SUBDIR: &str = "adi";
const DEFAULT_REGISTRY_URL: &str = "https://adi-plugin-registry.the-ihor.com";
const DEFAULT_SIGNALING_URL: &str = "wss://adi.the-ihor.com/api/signaling/ws";

/// ADI config directory ($ADI_CONFIG_DIR or ~/.config/adi)
pub fn config_dir() -> PathBuf {
    env_opt(EnvVar::AdiConfigDir.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(FALLBACK_CONFIG_DIR))
                .join(ADI_SUBDIR)
        })
}

/// ADI theme override ($ADI_THEME)
pub fn theme() -> Option<String> {
    env_opt(EnvVar::AdiTheme.as_str())
}

/// ADI language override ($ADI_LANG)
pub fn lang() -> Option<String> {
    env_opt(EnvVar::AdiLang.as_str())
}

/// System language ($LANG)
pub fn system_lang() -> Option<String> {
    env_opt(EnvVar::Lang.as_str())
}

/// Whether auto-install is disabled ($ADI_AUTO_INSTALL=false|0|no|off)
pub fn auto_install_disabled() -> bool {
    !env_bool_default_true(EnvVar::AdiAutoInstall.as_str())
}

/// Plugin registry URL ($ADI_REGISTRY_URL or default)
pub fn registry_url() -> String {
    env_or(EnvVar::AdiRegistryUrl.as_str(), DEFAULT_REGISTRY_URL)
}

/// Optional plugin registry URL override ($ADI_REGISTRY_URL)
pub fn registry_url_override() -> Option<String> {
    env_opt(EnvVar::AdiRegistryUrl.as_str())
}

/// Signaling server URL ($SIGNALING_SERVER_URL or default)
pub fn signaling_url() -> String {
    env_or(EnvVar::SignalingServerUrl.as_str(), DEFAULT_SIGNALING_URL)
}
