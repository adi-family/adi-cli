mod args;
mod cmd_external;
mod cmd_info;
mod cmd_interactive;
mod cmd_logs;
mod cmd_plugin;
mod cmd_run;
mod cmd_search;
mod cmd_start;
mod init;

use args::{Cli, Commands};
use clap::Parser;
use cli::completions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
        )
        .with_target(false)
        .init();

    tracing::trace!("ADI CLI starting");

    dotenvy::dotenv().ok();

    completions::ensure_completions_installed::<Cli>("adi");

    let cli = Cli::parse();
    tracing::trace!(lang = ?cli.lang, has_command = cli.command.is_some(), "CLI arguments parsed");

    init::initialize_i18n(cli.lang.as_deref()).await?;
    init::initialize_theme();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => {
            tracing::trace!("No command provided, entering interactive mode");
            match cmd_interactive::select_command().await {
                Some(cmd) => cmd,
                None => return Ok(()),
            }
        }
    };

    match command {
        Commands::SelfUpdate { force } => {
            tracing::trace!(force = force, "Dispatching: self-update");
            cli::self_update::self_update(force).await?
        }
        Commands::Start { port } => {
            tracing::trace!(port = port, "Dispatching: start");
            cmd_start::cmd_start(port).await?
        }
        Commands::Plugin { command } => {
            tracing::trace!("Dispatching: plugin");
            cmd_plugin::cmd_plugin(command).await?
        }
        Commands::Run { plugin_id, args } => {
            tracing::trace!(plugin_id = ?plugin_id, "Dispatching: run");
            cmd_run::cmd_run(plugin_id, args).await?
        }
        Commands::Logs {
            plugin_id,
            follow,
            lines,
            level,
            service,
        } => {
            tracing::trace!(plugin_id = %plugin_id, "Dispatching: logs");
            cmd_logs::cmd_logs(&plugin_id, follow, lines, level, service).await?
        }
        Commands::Info => {
            tracing::trace!("Dispatching: info");
            cmd_info::cmd_info().await?
        }
        Commands::External(args) => {
            tracing::trace!(args = ?args, "Dispatching: external");
            cmd_external::cmd_external(args).await?
        }
    }

    tracing::trace!("ADI CLI finished");
    Ok(())
}
