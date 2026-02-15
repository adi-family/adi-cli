use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, blocks::{List, Section, Renderable}, out_info};
use lib_i18n_core::t;

use crate::args::DebugCommands;

pub(crate) async fn cmd_debug(command: DebugCommands) -> anyhow::Result<()> {
    match command {
        DebugCommands::Services => cmd_services().await?,
    }
    Ok(())
}

async fn cmd_services() -> anyhow::Result<()> {
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    let plugins = runtime.list_installed();

    if plugins.is_empty() {
        out_info!("{}", t!("services-empty"));
        out_info!("{}", t!("services-hint"));
        return Ok(());
    }

    Section::new(t!("services-title")).print();

    List::new()
        .items(plugins.iter().map(|id| theme::brand_bold(id).to_string()))
        .print();

    Ok(())
}
