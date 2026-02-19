use std::path::PathBuf;

use lib_env_parse::{env_bool_default_true, env_opt, env_or, env_vars};

env_vars! {
    AdiConfigDir       => "ADI_CONFIG_DIR",
    AdiTheme           => "ADI_THEME",
    AdiLang            => "ADI_LANG",
    AdiPowerUser       => "ADI_POWER_USER",
    Lang               => "LANG",
    AdiAutoInstall     => "ADI_AUTO_INSTALL",
    AdiRegistryUrl     => "ADI_REGISTRY_URL",
    SignalingServerUrl  => "SIGNALING_SERVER_URL",
    // Daemon env vars
    AdiDaemonSocket    => "ADI_DAEMON_SOCKET",
    AdiDaemonPid       => "ADI_DAEMON_PID",
    AdiDaemonLog       => "ADI_DAEMON_LOG",
    AdiUser            => "ADI_USER",
    AdiRootUser        => "ADI_ROOT_USER",
    AdiDaemonTcpPort   => "ADI_DAEMON_TCP_PORT",
}

const FALLBACK_CONFIG_DIR: &str = "~/.config";
const ADI_SUBDIR: &str = "adi";
const DEFAULT_REGISTRY_URL: &str = "https://adi-plugin-registry.the-ihor.com";
const DEFAULT_SIGNALING_URL: &str = "wss://adi.the-ihor.com/api/signaling/ws";
pub const CLI_PLUGIN_PREFIX: &str = "adi.cli.";

/// ADI config directory ($ADI_CONFIG_DIR or ~/.config/adi)
pub fn config_dir() -> PathBuf {
    let dir = env_opt(EnvVar::AdiConfigDir.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(FALLBACK_CONFIG_DIR))
                .join(ADI_SUBDIR)
        });
    tracing::trace!(dir = %dir.display(), "Resolved config directory");
    dir
}

/// ADI theme override ($ADI_THEME)
pub fn theme() -> Option<String> {
    let val = env_opt(EnvVar::AdiTheme.as_str());
    tracing::trace!(value = ?val, "ADI_THEME env var");
    val
}

/// ADI language override ($ADI_LANG)
pub fn lang() -> Option<String> {
    let val = env_opt(EnvVar::AdiLang.as_str());
    tracing::trace!(value = ?val, "ADI_LANG env var");
    val
}

/// System language ($LANG)
pub fn system_lang() -> Option<String> {
    let val = env_opt(EnvVar::Lang.as_str());
    tracing::trace!(value = ?val, "LANG env var");
    val
}

/// Power user mode from env var ($ADI_POWER_USER)
pub fn power_user_env() -> Option<bool> {
    let val = env_opt(EnvVar::AdiPowerUser.as_str());
    let result = val.as_ref().map(|v| lib_env_parse::is_truthy(v));
    tracing::trace!(value = ?result, "ADI_POWER_USER env var");
    result
}

/// Check if power user mode is enabled (env var > config > default false)
pub fn is_power_user() -> bool {
    // Priority: env var > config > default (false)
    if let Some(env_val) = power_user_env() {
        return env_val;
    }

    if let Ok(config) = crate::user_config::UserConfig::load() {
        if let Some(power_user) = config.power_user {
            return power_user;
        }
    }

    false
}

/// Whether auto-install is disabled ($ADI_AUTO_INSTALL=false|0|no|off)
pub fn auto_install_disabled() -> bool {
    let disabled = !env_bool_default_true(EnvVar::AdiAutoInstall.as_str());
    tracing::trace!(disabled = disabled, "Auto-install disabled check");
    disabled
}

/// Plugin registry URL ($ADI_REGISTRY_URL or default)
pub fn registry_url() -> String {
    let url = env_or(EnvVar::AdiRegistryUrl.as_str(), DEFAULT_REGISTRY_URL);
    tracing::trace!(url = %url, "Registry URL");
    url
}

/// Optional plugin registry URL override ($ADI_REGISTRY_URL)
pub fn registry_url_override() -> Option<String> {
    let val = env_opt(EnvVar::AdiRegistryUrl.as_str());
    tracing::trace!(value = ?val, "Registry URL override");
    val
}

/// Signaling server URL ($SIGNALING_SERVER_URL or default)
pub fn signaling_url() -> String {
    let url = env_or(EnvVar::SignalingServerUrl.as_str(), DEFAULT_SIGNALING_URL);
    tracing::trace!(url = %url, "Signaling URL");
    url
}

// ============================================================================
// Daemon configuration
// ============================================================================

const DEFAULT_DAEMON_USER: &str = "adi";
const DEFAULT_DAEMON_ROOT_USER: &str = "adi-root";
const DEFAULT_DAEMON_TCP_PORT: u16 = 14731;

/// ADI data directory (~/.local/share/adi)
pub fn data_dir() -> PathBuf {
    let dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("~/.local/share"))
        .join(ADI_SUBDIR);
    tracing::trace!(dir = %dir.display(), "Resolved data directory");
    dir
}

/// Plugins directory (~/.local/share/adi/plugins)
pub fn plugins_dir() -> PathBuf {
    data_dir().join("plugins")
}

/// Daemon socket path ($ADI_DAEMON_SOCKET or ~/.local/share/adi/daemon.sock)
pub fn daemon_socket_path() -> PathBuf {
    let path = env_opt(EnvVar::AdiDaemonSocket.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir().join("daemon.sock"));
    tracing::trace!(path = %path.display(), "Daemon socket path");
    path
}

/// Daemon PID file path ($ADI_DAEMON_PID or ~/.local/share/adi/daemon.pid)
pub fn daemon_pid_path() -> PathBuf {
    let path = env_opt(EnvVar::AdiDaemonPid.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir().join("daemon.pid"));
    tracing::trace!(path = %path.display(), "Daemon PID path");
    path
}

/// Daemon log file path ($ADI_DAEMON_LOG or ~/.local/share/adi/logs/daemon.log)
pub fn daemon_log_path() -> PathBuf {
    let path = env_opt(EnvVar::AdiDaemonLog.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| data_dir().join("logs").join("daemon.log"));
    tracing::trace!(path = %path.display(), "Daemon log path");
    path
}

/// Regular daemon user ($ADI_USER or "adi")
pub fn daemon_user() -> String {
    let user = env_or(EnvVar::AdiUser.as_str(), DEFAULT_DAEMON_USER);
    tracing::trace!(user = %user, "Daemon user");
    user
}

/// Privileged daemon user ($ADI_ROOT_USER or "adi-root")
pub fn daemon_root_user() -> String {
    let user = env_or(EnvVar::AdiRootUser.as_str(), DEFAULT_DAEMON_ROOT_USER);
    tracing::trace!(user = %user, "Daemon root user");
    user
}

/// Daemon TCP port for non-Unix platforms ($ADI_DAEMON_TCP_PORT or 14731)
pub fn daemon_tcp_port() -> u16 {
    env_opt(EnvVar::AdiDaemonTcpPort.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_DAEMON_TCP_PORT)
}
