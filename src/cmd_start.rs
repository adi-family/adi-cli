use cli::plugin_registry::PluginManager;
use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, out_info, out_success};

pub(crate) async fn cmd_start(port: u16) -> anyhow::Result<()> {
    tracing::trace!(port = port, "cmd_start invoked");

    let manager = PluginManager::new();

    if manager.is_installed("adi.cocoon").is_none() {
        tracing::trace!("Cocoon plugin not installed, installing");
        out_info!("{}", theme::muted("Installing cocoon plugin..."));
        manager.install_plugin("adi.cocoon", None).await?;
        out_success!("Cocoon plugin installed!");
    }

    tracing::trace!("Loading cocoon plugin for setup");
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.scan_and_load_plugin("adi.cocoon").await?;

    let context = serde_json::json!({
        "command": "adi.cocoon",
        "args": ["setup", "--port", port.to_string()],
        "cwd": std::env::current_dir().unwrap_or_default().to_string_lossy()
    });

    runtime.run_cli_command("adi.cocoon", &context.to_string()).await?;

    Ok(())
}
