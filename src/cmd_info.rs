use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::blocks::{KeyValue, Renderable, Section};
use lib_console_output::theme;
use lib_i18n_core::t;

pub(crate) async fn cmd_info() -> anyhow::Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let config_dir = cli::clienv::config_dir();
    let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();
    let registry_url = cli::clienv::registry_url();
    let active_theme = lib_console_output::theme::active();
    let lang = cli::clienv::lang()
        .or_else(cli::clienv::system_lang)
        .unwrap_or_else(|| "en-US".to_string());

    Section::new(t!("info-title")).width(50).print();

    KeyValue::new()
        .entry(t!("info-version"), theme::brand_bold(format!("v{version}")).to_string())
        .entry(t!("info-config-dir"), theme::muted(config_dir.display()).to_string())
        .entry(t!("info-plugins-dir"), theme::muted(plugins_dir.display()).to_string())
        .entry(t!("info-registry"), theme::muted(&registry_url).to_string())
        .entry(t!("info-theme"), theme::brand(&active_theme.name).to_string())
        .entry(t!("info-language"), theme::muted(&lang).to_string())
        .print();

    println!();

    print_installed_plugins(&plugins_dir).await;
    print_available_commands().await;

    Ok(())
}

async fn print_installed_plugins(plugins_dir: &std::path::Path) {
    let plugin_dirs: Vec<String> = std::fs::read_dir(plugins_dir)
        .ok()
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| {
            e.path().is_dir()
                && e.file_name() != lib_plugin_host::command_index::COMMANDS_DIR_NAME
        })
        .filter_map(|e| Some(e.file_name().to_str()?.to_string()))
        .collect();

    Section::new(t!("info-installed-plugins", "count" => plugin_dirs.len().to_string()))
        .width(50)
        .print();

    if plugin_dirs.is_empty() {
        lib_console_output::fg_println!("  {}", theme::muted(t!("info-no-plugins")));
    } else {
        for id in &plugin_dirs {
            lib_console_output::fg_println!("  {} {}", theme::brand(theme::icons::BRAND), theme::foreground(id));
        }
    }

    println!();
}

async fn print_available_commands() {
    Section::new(t!("info-commands-title")).width(50).print();

    let builtins = [
        ("info", t!("info-cmd-info")),
        ("start", t!("info-cmd-start")),
        ("plugin", t!("info-cmd-plugin")),
        ("run", t!("info-cmd-run")),
        ("logs", t!("info-cmd-logs")),
        ("self-update", t!("info-cmd-self-update")),
    ];

    for (name, desc) in &builtins {
        lib_console_output::fg_println!(
            "  {} {:<16} {}",
            theme::brand(theme::icons::BRAND),
            theme::bold(name),
            theme::muted(desc),
        );
    }

    if let Ok(runtime) = PluginRuntime::new(RuntimeConfig::default()).await {
        let plugin_commands = runtime.discover_cli_commands();
        if !plugin_commands.is_empty() {
            println!();
            lib_console_output::fg_println!("  {}", theme::muted(t!("info-plugin-commands")));
            for cmd in &plugin_commands {
                let aliases = if cmd.aliases.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", cmd.aliases.join(", "))
                };
                lib_console_output::fg_println!(
                    "  {} {:<16} {}{}",
                    theme::brand(theme::icons::BRAND),
                    theme::bold(&cmd.command),
                    theme::muted(&cmd.description),
                    theme::muted(aliases),
                );
            }
        }
    }
}
