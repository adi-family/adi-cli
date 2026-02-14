use cli::plugin_registry::PluginManager;
use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, blocks::{Columns, Section, Renderable}, out_info, out_warn, out_error, out_success};
use lib_i18n_core::{t, LocalizedError};

use super::run::handle_cli_result;

pub(crate) async fn cmd_external(args: Vec<String>) -> anyhow::Result<()> {
    if args.is_empty() {
        out_error!("{} {}", t!("common-error-prefix"), t!("external-error-no-command"));
        std::process::exit(1);
    }

    let command = args[0].clone();
    let cmd_args: Vec<String> = args.into_iter().skip(1).collect();

    let mut runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

    // Discover CLI commands (fast - only reads manifests, no binary loading)
    let cli_commands = runtime.discover_cli_commands();

    let matching_plugin = cli_commands
        .iter()
        .find(|c| c.command == command || c.aliases.contains(&command));

    let plugin_id = if let Some(plugin_cmd) = matching_plugin {
        plugin_cmd.plugin_id.clone()
    } else {
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

/// Result of plugin auto-installation attempt.
enum AutoinstallResult {
    /// Plugin was installed successfully, contains the plugin ID
    Installed(String),
    /// No plugin found in registry providing this command
    NotFound,
    /// User declined installation
    Declined,
    /// Installation failed
    Failed,
}

/// Try to auto-install a plugin that provides the given command.
///
/// The plugin ID is inferred from the command name using the pattern `adi.cli.{command}`.
async fn try_autoinstall_plugin(
    command: &str,
    cli_commands: &[cli::plugin_runtime::PluginCliCommand],
) -> AutoinstallResult {
    use std::io::{self, Write};

    let auto_install_disabled = cli::clienv::auto_install_disabled();

    let plugin_id = format!("adi.cli.{}", command);

    let manager = PluginManager::new();

    match manager.get_plugin_info(&plugin_id).await {
        Ok(Some(_info)) => {
            out_info!("{}", t!("external-autoinstall-found", "id" => &plugin_id, "command" => command));

            if auto_install_disabled {
                out_warn!("{}", t!("external-autoinstall-disabled", "id" => &plugin_id));
                return AutoinstallResult::Declined;
            }

            let is_interactive = atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout);

            let should_install = if is_interactive {
                print!("{} ", t!("external-autoinstall-prompt"));
                io::stdout().flush().ok();

                let mut input = String::new();
                if io::stdin().read_line(&mut input).is_ok() {
                    let input = input.trim().to_lowercase();
                    input == "y" || input == "yes"
                } else {
                    false
                }
            } else {
                true
            };

            if !should_install {
                out_warn!("{}", t!("external-autoinstall-disabled", "id" => &plugin_id));
                return AutoinstallResult::Declined;
            }

            out_info!("{}", t!("external-autoinstall-installing", "id" => &plugin_id));

            match manager.install_with_dependencies(&plugin_id, None).await {
                Ok(()) => {
                    out_success!("{} {}", t!("common-success-prefix"), t!("external-autoinstall-success"));
                    AutoinstallResult::Installed(plugin_id)
                }
                Err(e) => {
                    out_error!("{} {}", t!("common-error-prefix"), t!("external-autoinstall-failed", "error" => &e.localized()));
                    AutoinstallResult::Failed
                }
            }
        }
        Ok(None) | Err(_) => {
            out_error!("{} {}", t!("common-error-prefix"), t!("external-error-unknown", "command" => command));
            out_info!("{}", t!("external-autoinstall-not-found", "command" => command));

            if cli_commands.is_empty() {
                out_info!("{}", t!("external-error-no-installed"));
                out_info!("{}", t!("external-hint-install"));
            } else {
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

            AutoinstallResult::NotFound
        }
    }
}
