use cli::completions;
use cli::plugin_registry::PluginManager;
use dialoguer::{theme::ColorfulTheme, Confirm};
use lib_console_output::{theme, blocks::{Columns, Section, Renderable}, out_info, out_warn, out_error, out_success};
use lib_i18n_core::{t, LocalizedError};

use crate::args::{Cli, PluginCommands};

pub(crate) async fn cmd_plugin(command: PluginCommands) -> anyhow::Result<()> {
    let manager = PluginManager::new();

    match command {
        PluginCommands::Search { query } => {
            super::search::cmd_search(&query).await?;
        }
        PluginCommands::List => {
            Section::new(t!("plugin-list-title")).print();

            let plugins = manager.list_plugins().await?;

            if plugins.is_empty() {
                out_info!("{}", t!("plugin-list-empty"));
                return Ok(());
            }

            let mut cols = Columns::new()
                .header(["Plugin", "Version", "Description", "Type"]);
            for plugin in &plugins {
                cols = cols.row([
                    theme::brand_bold(&plugin.id).to_string(),
                    theme::muted(format!("v{}", plugin.latest_version)).to_string(),
                    plugin.description.clone(),
                    theme::warning(&plugin.plugin_type).to_string(),
                ]);
            }
            cols.print();

            for plugin in &plugins {
                if !plugin.tags.is_empty() {
                    out_info!("{}: Tags: {}", theme::brand(&plugin.id), theme::muted(plugin.tags.join(", ")));
                }
            }
        }
        PluginCommands::Installed => {
            Section::new(t!("plugin-installed-title")).print();

            let installed = manager.list_installed().await?;

            if installed.is_empty() {
                out_info!("{}", t!("plugin-installed-empty"));
                out_info!("{}", t!("plugin-installed-hint"));
                return Ok(());
            }

            let cols = Columns::new()
                .header(["Plugin", "Version"])
                .rows(installed.iter().map(|(id, version)| [
                    theme::brand_bold(id).to_string(),
                    theme::muted(format!("v{}", version)).to_string(),
                ]));
            cols.print();
        }
        PluginCommands::Install { plugin_id, version } => {
            manager
                .install_plugins_matching(&plugin_id, version.as_deref())
                .await?;
            regenerate_completions_quiet();
        }
        PluginCommands::Update { plugin_id } => {
            manager.update_plugin(&plugin_id).await?;
            regenerate_completions_quiet();
        }
        PluginCommands::UpdateAll => {
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
        }
        PluginCommands::Uninstall { plugin_id } => {
            let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(t!("plugin-uninstall-prompt", "id" => &plugin_id))
                .default(false)
                .interact()?;

            if !confirmed {
                out_info!("{}", t!("plugin-uninstall-cancelled"));
                return Ok(());
            }

            manager.uninstall_plugin(&plugin_id).await?;
            regenerate_completions_quiet();
        }
        PluginCommands::Path { plugin_id } => {
            let plugin_dir = manager.plugin_path(&plugin_id);
            let version_file = plugin_dir.join(".version");

            if !version_file.exists() {
                out_error!("Plugin {} is not installed", theme::brand(&plugin_id));
                std::process::exit(1);
            }

            let version = tokio::fs::read_to_string(&version_file).await?;
            let version = version.trim();
            let versioned_path = plugin_dir.join(version);

            // Print just the path (useful for scripting)
            println!("{}", versioned_path.display());
        }
    }

    Ok(())
}

/// Regenerate shell completions silently (called after plugin changes).
fn regenerate_completions_quiet() {
    if let Err(e) = completions::regenerate_completions::<Cli>("adi") {
        #[cfg(debug_assertions)]
        out_warn!("Failed to regenerate completions: {}", e);
        let _ = e;
    }
}
