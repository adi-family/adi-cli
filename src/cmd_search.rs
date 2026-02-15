use cli::plugin_registry::PluginManager;
use lib_console_output::{theme, blocks::{Columns, Section, Renderable}, out_info};
use lib_i18n_core::t;

pub(crate) async fn cmd_search(query: &str) -> anyhow::Result<()> {
    tracing::trace!(query = %query, "cmd_search invoked");
    let manager = PluginManager::new();

    out_info!("{}", t!("search-searching", "query" => query));

    let results = manager.search(query).await?;
    tracing::trace!(packages = results.packages.len(), plugins = results.plugins.len(), "Search results received");

    if results.packages.is_empty() && results.plugins.is_empty() {
        out_info!("{}", t!("search-no-results"));
        return Ok(());
    }

    if !results.packages.is_empty() {
        Section::new(t!("search-packages-title")).print();
        let cols = Columns::new()
            .header(["Package", "Version", "Description"])
            .rows(results.packages.iter().map(|pkg| [
                theme::brand_bold(&pkg.id).to_string(),
                theme::muted(format!("v{}", pkg.latest_version)).to_string(),
                pkg.description.clone(),
            ]));
        cols.print();

        for pkg in &results.packages {
            if !pkg.tags.is_empty() {
                out_info!("{}: Tags: {}", theme::brand(&pkg.id), theme::muted(pkg.tags.join(", ")));
            }
        }
    }

    if !results.plugins.is_empty() {
        Section::new(t!("search-plugins-title")).print();
        let cols = Columns::new()
            .header(["Plugin", "Version", "Description", "Type"])
            .rows(results.plugins.iter().map(|plugin| [
                theme::brand_bold(&plugin.id).to_string(),
                theme::muted(format!("v{}", plugin.latest_version)).to_string(),
                plugin.description.clone(),
                theme::warning(&plugin.plugin_type).to_string(),
            ]));
        cols.print();

        for plugin in &results.plugins {
            if !plugin.tags.is_empty() {
                out_info!("{}: Tags: {}", theme::brand(&plugin.id), theme::muted(plugin.tags.join(", ")));
            }
        }
    }

    out_info!("{}", t!("search-results-summary",
        "packages" => &results.packages.len().to_string(),
        "plugins" => &results.plugins.len().to_string()
    ));

    Ok(())
}
