use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::input::{Confirm, Input, Select, SelectOption};
use lib_i18n_core::t;

use crate::args::{Commands, PluginCommands};

#[derive(Clone)]
enum CommandEntry {
    Builtin(BuiltinCommand),
    Plugin { command: String },
}

#[derive(Clone, Copy)]
enum BuiltinCommand {
    SelfUpdate,
    Start,
    Plugin,
    Run,
    Logs,
}

/// Returns `None` if user cancels.
pub(crate) async fn select_command() -> Option<Commands> {
    tracing::trace!("Entering interactive command selection");
    let mut options: Vec<SelectOption<CommandEntry>> = vec![
        SelectOption::new(
            t!("interactive-cmd-start"),
            CommandEntry::Builtin(BuiltinCommand::Start),
        )
        .with_description(t!("interactive-cmd-start-desc")),
        SelectOption::new(
            t!("interactive-cmd-plugin"),
            CommandEntry::Builtin(BuiltinCommand::Plugin),
        )
        .with_description(t!("interactive-cmd-plugin-desc")),
        SelectOption::new(
            t!("interactive-cmd-run"),
            CommandEntry::Builtin(BuiltinCommand::Run),
        )
        .with_description(t!("interactive-cmd-run-desc")),
        SelectOption::new(
            t!("interactive-cmd-logs"),
            CommandEntry::Builtin(BuiltinCommand::Logs),
        )
        .with_description(t!("interactive-cmd-logs-desc")),
        SelectOption::new(
            t!("interactive-cmd-self-update"),
            CommandEntry::Builtin(BuiltinCommand::SelfUpdate),
        )
        .with_description(t!("interactive-cmd-self-update-desc")),
    ];
    if let Ok(runtime) = PluginRuntime::new(RuntimeConfig::default()).await {
        let plugin_commands = runtime.discover_cli_commands();
        tracing::trace!(count = plugin_commands.len(), "Discovered plugin commands for interactive menu");
        for cmd in plugin_commands {
            options.push(
                SelectOption::new(
                    cmd.command.clone(),
                    CommandEntry::Plugin {
                        command: cmd.command.clone(),
                    },
                )
                .with_description(cmd.description.clone()),
            );
        }
    }

    let entry = Select::new(t!("interactive-select-command"))
        .options(options)
        .filterable(true)
        .max_display(Some(15))
        .run()?;

    match &entry {
        CommandEntry::Builtin(_) => tracing::trace!("User selected builtin command"),
        CommandEntry::Plugin { command } => tracing::trace!(command = %command, "User selected plugin command"),
    }

    match entry {
        CommandEntry::Builtin(cmd) => prompt_builtin_args(cmd),
        CommandEntry::Plugin { command } => Some(Commands::External(vec![command])),
    }
}

fn prompt_builtin_args(cmd: BuiltinCommand) -> Option<Commands> {
    match cmd {
        BuiltinCommand::SelfUpdate => prompt_self_update(),
        BuiltinCommand::Start => prompt_start(),
        BuiltinCommand::Plugin => prompt_plugin(),
        BuiltinCommand::Run => Some(Commands::Run {
            plugin_id: None,
            args: vec![],
        }),
        BuiltinCommand::Logs => prompt_logs(),
    }
}

fn prompt_self_update() -> Option<Commands> {
    let force = Confirm::new(t!("interactive-self-update-force"))
        .default(false)
        .run()
        .unwrap_or(false);
    Some(Commands::SelfUpdate { force })
}

fn prompt_start() -> Option<Commands> {
    let port_str = Input::new(t!("interactive-start-port"))
        .default("14730")
        .run()?;
    let port = port_str.parse::<u16>().unwrap_or(14730);
    Some(Commands::Start { port })
}

fn prompt_plugin() -> Option<Commands> {
    let subcmd = Select::new(t!("interactive-plugin-select"))
        .items([
            (t!("interactive-plugin-list"), "list"),
            (t!("interactive-plugin-installed"), "installed"),
            (t!("interactive-plugin-search"), "search"),
            (t!("interactive-plugin-install"), "install"),
            (t!("interactive-plugin-update"), "update"),
            (t!("interactive-plugin-update-all"), "update-all"),
            (t!("interactive-plugin-uninstall"), "uninstall"),
            (t!("interactive-plugin-path"), "path"),
        ])
        .run()?;

    match subcmd {
        "list" => Some(Commands::Plugin {
            command: PluginCommands::List,
        }),
        "installed" => Some(Commands::Plugin {
            command: PluginCommands::Installed,
        }),
        "search" => {
            let query = Input::new(t!("interactive-search-query")).required().run()?;
            Some(Commands::Plugin {
                command: PluginCommands::Search { query },
            })
        }
        "install" => {
            let plugin_id = Input::new(t!("interactive-plugin-install-id"))
                .required()
                .run()?;
            Some(Commands::Plugin {
                command: PluginCommands::Install {
                    plugin_id,
                    version: None,
                },
            })
        }
        "update" => {
            let plugin_id = Input::new(t!("interactive-plugin-update-id"))
                .required()
                .run()?;
            Some(Commands::Plugin {
                command: PluginCommands::Update { plugin_id },
            })
        }
        "update-all" => Some(Commands::Plugin {
            command: PluginCommands::UpdateAll,
        }),
        "uninstall" => {
            let plugin_id = Input::new(t!("interactive-plugin-uninstall-id"))
                .required()
                .run()?;
            Some(Commands::Plugin {
                command: PluginCommands::Uninstall { plugin_id },
            })
        }
        "path" => {
            let plugin_id = Input::new(t!("interactive-plugin-path-id"))
                .required()
                .run()?;
            Some(Commands::Plugin {
                command: PluginCommands::Path { plugin_id },
            })
        }
        _ => None,
    }
}

fn prompt_logs() -> Option<Commands> {
    let plugin_id = Input::new(t!("interactive-logs-plugin-id"))
        .required()
        .run()?;
    let follow = Confirm::new(t!("interactive-logs-follow"))
        .default(false)
        .run()
        .unwrap_or(false);
    let lines_str = Input::new(t!("interactive-logs-lines"))
        .default("50")
        .run()
        .unwrap_or_else(|| "50".to_string());
    let lines = lines_str.parse::<u32>().unwrap_or(50);

    Some(Commands::Logs {
        plugin_id,
        follow,
        lines,
        level: None,
        service: None,
    })
}
