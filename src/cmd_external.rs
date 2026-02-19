use cli::plugin_registry::PluginManager;
use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, blocks::{Columns, Section, Renderable}, out_info, out_warn, out_error, out_success};
use lib_i18n_core::{t, LocalizedError};

use crate::cmd_run::handle_cli_result;

pub(crate) async fn cmd_external(args: Vec<String>) -> anyhow::Result<()> {
    tracing::trace!(args = ?args, "Handling external plugin command");

    if args.is_empty() {
        out_error!("{} {}", t!("common-error-prefix"), t!("external-error-no-command"));
        std::process::exit(1);
    }

    let command = args[0].clone();
    let cmd_args: Vec<String> = args.into_iter().skip(1).collect();
    tracing::trace!(command = %command, cmd_args = ?cmd_args, "Parsed external command");

    let mut runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

    let cli_commands = runtime.discover_cli_commands();

    let matching_plugin = cli_commands
        .iter()
        .find(|c| c.command == command || c.aliases.contains(&command));

    let plugin_id = if let Some(plugin_cmd) = matching_plugin {
        tracing::trace!(plugin_id = %plugin_cmd.plugin_id, "Found matching plugin for command");
        plugin_cmd.plugin_id.clone()
    } else {
        tracing::trace!(command = %command, "No installed plugin found, trying auto-install");
        match try_autoinstall_plugin(&command, &cli_commands).await {
            AutoinstallResult::Installed(id) => {
                runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
                id
            }
            AutoinstallResult::NotFound
            | AutoinstallResult::Declined
            | AutoinstallResult::Failed => {
                std::process::exit(1);
            }
        }
    };

    if let Err(e) = runtime.scan_and_load_plugin(&plugin_id).await {
        out_error!("{} {}", t!("common-error-prefix"), t!("external-error-load-failed", "id" => &plugin_id, "error" => &e.localized()));
        out_info!("{}", t!("external-hint-reinstall", "id" => &plugin_id));
        std::process::exit(1);
    }

    let context = serde_json::json!({
        "command": &plugin_id,
        "args": cmd_args,
        "cwd": std::env::current_dir()?.to_string_lossy()
    });

    match runtime.run_cli_command(&plugin_id, &context.to_string()).await {
        Ok(result) => {
            handle_cli_result(&result);
            Ok(())
        }
        Err(e) => {
            out_error!("{} {}", t!("common-error-prefix"), t!("external-error-run-failed", "command" => &command, "error" => &e.localized()));
            std::process::exit(1);
        }
    }
}

enum AutoinstallResult {
    Installed(String),
    NotFound,
    Declined,
    Failed,
}

async fn try_autoinstall_plugin(
    command: &str,
    cli_commands: &[cli::plugin_runtime::PluginCliCommand],
) -> AutoinstallResult {
    let plugin_id = format!("{}{}", cli::clienv::CLI_PLUGIN_PREFIX, command);
    tracing::trace!(command = %command, plugin_id = %plugin_id, "Attempting auto-install");

    let manager = PluginManager::new();

    match manager.get_plugin_info(&plugin_id).await {
        Ok(Some(_info)) => prompt_and_install(&manager, command, &plugin_id).await,
        Ok(None) | Err(_) => {
            show_unknown_command(command, cli_commands);
            AutoinstallResult::NotFound
        }
    }
}

async fn prompt_and_install(manager: &PluginManager, command: &str, plugin_id: &str) -> AutoinstallResult {
    tracing::trace!(plugin_id = %plugin_id, "Plugin found in registry");
    out_info!("{}", t!("external-autoinstall-found", "id" => plugin_id, "command" => command));

    if cli::clienv::auto_install_disabled() {
        out_warn!("{}", t!("external-autoinstall-disabled", "id" => plugin_id));
        return AutoinstallResult::Declined;
    }

    let is_interactive = std::io::IsTerminal::is_terminal(&std::io::stdin())
        && std::io::IsTerminal::is_terminal(&std::io::stdout());

    if is_interactive && !confirm_install() {
        out_warn!("{}", t!("external-autoinstall-disabled", "id" => plugin_id));
        return AutoinstallResult::Declined;
    }

    out_info!("{}", t!("external-autoinstall-installing", "id" => plugin_id));

    match manager.install_with_dependencies(plugin_id, None).await {
        Ok(()) => {
            out_success!("{} {}", t!("common-success-prefix"), t!("external-autoinstall-success"));
            AutoinstallResult::Installed(plugin_id.to_string())
        }
        Err(e) => {
            out_error!("{} {}", t!("common-error-prefix"), t!("external-autoinstall-failed", "error" => &e.localized()));
            AutoinstallResult::Failed
        }
    }
}

fn confirm_install() -> bool {
    use std::io::{self, Write};

    print!("{} ", t!("external-autoinstall-prompt"));
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).is_ok() && matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn show_unknown_command(command: &str, cli_commands: &[cli::plugin_runtime::PluginCliCommand]) {
    out_error!("{} {}", t!("common-error-prefix"), t!("external-error-unknown", "command" => command));
    out_info!("{}", t!("external-autoinstall-not-found", "command" => command));

    if cli_commands.is_empty() {
        out_info!("{}", t!("external-error-no-installed"));
        out_info!("{}", t!("external-hint-install"));
        return;
    }

    Section::new(t!("external-available-title")).print();
    Columns::new()
        .header(["Command", "Description"])
        .rows(cli_commands.iter().map(|cmd| {
            let desc = if cmd.aliases.is_empty() {
                cmd.description.clone()
            } else {
                format!("{}{}", cmd.description, theme::muted(format!(" (aliases: {})", cmd.aliases.join(", "))))
            };
            [theme::brand_bold(&cmd.command).to_string(), desc]
        }))
        .print();
}
