use cli::completions;
use cli::plugin_registry::PluginManager;
use lib_console_output::{theme, blocks::{Columns, Section, Renderable}, out_info, out_warn, out_error, out_success};
use lib_console_output::input::Confirm;
use lib_i18n_core::{t, LocalizedError};

use crate::args::{Cli, PluginCommands};

pub(crate) async fn cmd_plugin(command: PluginCommands) -> anyhow::Result<()> {
    tracing::trace!("cmd_plugin invoked");
    let manager = PluginManager::new();

    match command {
        PluginCommands::Search { query } => handle_search(&query).await,
        PluginCommands::List => handle_list(&manager).await,
        PluginCommands::Installed => handle_installed(&manager).await,
        PluginCommands::Install { plugin_id, version } => {
            handle_install(&manager, &plugin_id, version.as_deref()).await
        }
        PluginCommands::Update { plugin_id } => handle_update(&manager, &plugin_id).await,
        PluginCommands::UpdateAll => handle_update_all(&manager).await,
        PluginCommands::Uninstall { plugin_id } => handle_uninstall(&manager, &plugin_id).await,
        PluginCommands::Path { plugin_id } => handle_path(&manager, &plugin_id).await,
    }
}

async fn handle_search(query: &str) -> anyhow::Result<()> {
    tracing::trace!(query = %query, "Searching plugins");
    crate::cmd_search::cmd_search(query).await
}

async fn handle_list(manager: &PluginManager) -> anyhow::Result<()> {
    tracing::trace!("Listing available plugins");
    Section::new(t!("plugin-list-title")).print();

    let plugins = manager.list_plugins().await?;

    if plugins.is_empty() {
        out_info!("{}", t!("plugin-list-empty"));
        return Ok(());
    }

    Columns::new()
        .header(["Plugin", "Version", "Description", "Type"])
        .rows(plugins.iter().map(|p| [
            theme::brand_bold(&p.id).to_string(),
            theme::muted(format!("v{}", p.latest_version)).to_string(),
            p.description.clone(),
            theme::warning(&p.plugin_type).to_string(),
        ]))
        .print();

    for plugin in &plugins {
        if !plugin.tags.is_empty() {
            out_info!("{}: Tags: {}", theme::brand(&plugin.id), theme::muted(plugin.tags.join(", ")));
        }
    }

    Ok(())
}

async fn handle_installed(manager: &PluginManager) -> anyhow::Result<()> {
    tracing::trace!("Listing installed plugins");
    Section::new(t!("plugin-installed-title")).print();

    let installed = manager.list_installed().await?;

    if installed.is_empty() {
        out_info!("{}", t!("plugin-installed-empty"));
        out_info!("{}", t!("plugin-installed-hint"));
        return Ok(());
    }

    Columns::new()
        .header(["Plugin", "Version"])
        .rows(installed.iter().map(|(id, version)| [
            theme::brand_bold(id).to_string(),
            theme::muted(format!("v{}", version)).to_string(),
        ]))
        .print();

    Ok(())
}

async fn handle_install(manager: &PluginManager, plugin_id: &str, version: Option<&str>) -> anyhow::Result<()> {
    tracing::trace!(plugin_id = %plugin_id, version = ?version, "Installing plugin");
    manager.install_plugins_matching(plugin_id, version).await?;
    regenerate_completions_quiet();
    Ok(())
}

async fn handle_update(manager: &PluginManager, plugin_id: &str) -> anyhow::Result<()> {
    tracing::trace!(plugin_id = %plugin_id, "Updating plugin");
    manager.update_plugin(plugin_id).await?;
    regenerate_completions_quiet();
    Ok(())
}

async fn handle_update_all(manager: &PluginManager) -> anyhow::Result<()> {
    tracing::trace!("Updating all plugins");
    let installed = manager.list_installed().await?;

    if installed.is_empty() {
        out_info!("{}", t!("plugin-list-empty"));
        return Ok(());
    }

    out_info!("{}", t!("plugin-update-all-start", "count" => &installed.len().to_string()));

    for (id, _) in installed {
        if let Err(e) = manager.update_plugin(&id).await {
            out_warn!("{}", t!("plugin-update-all-warning", "id" => &id, "error" => &e.localized()));
        }
    }

    out_success!("{}", t!("plugin-update-all-done"));
    regenerate_completions_quiet();
    Ok(())
}

async fn handle_uninstall(manager: &PluginManager, plugin_id: &str) -> anyhow::Result<()> {
    tracing::trace!(plugin_id = %plugin_id, "Uninstalling plugin");
    let confirmed = Confirm::new(t!("plugin-uninstall-prompt", "id" => plugin_id))
        .default(false)
        .run()
        .unwrap_or(false);

    if !confirmed {
        out_info!("{}", t!("plugin-uninstall-cancelled"));
        return Ok(());
    }

    manager.uninstall_plugin(plugin_id).await?;
    regenerate_completions_quiet();
    Ok(())
}

async fn handle_path(manager: &PluginManager, plugin_id: &str) -> anyhow::Result<()> {
    tracing::trace!(plugin_id = %plugin_id, "Resolving plugin path");
    let plugin_dir = manager.plugin_path(plugin_id);
    let version_file = plugin_dir.join(".version");

    if !version_file.exists() {
        out_error!("Plugin {} is not installed", theme::brand(plugin_id));
        std::process::exit(1);
    }

    let version = tokio::fs::read_to_string(&version_file).await?;
    let versioned_path = plugin_dir.join(version.trim());
    println!("{}", versioned_path.display());
    Ok(())
}

fn regenerate_completions_quiet() {
    if let Err(e) = completions::regenerate_completions::<Cli>("adi") {
        #[cfg(debug_assertions)]
        out_warn!("Failed to regenerate completions: {}", e);
        let _ = e;
    }
}
