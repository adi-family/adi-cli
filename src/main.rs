mod args;
mod cmd_completions;
mod cmd_debug;
mod cmd_external;
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
    dotenvy::dotenv().ok();

    // Auto-initialize completions on first run
    completions::ensure_completions_installed::<Cli>("adi");

    let cli = Cli::parse();

    init::initialize_i18n(cli.lang.as_deref()).await?;
    init::initialize_theme();

    let command = match cli.command {
        Some(cmd) => cmd,
        None => match cmd_interactive::select_command().await {
            Some(cmd) => cmd,
            None => return Ok(()),
        },
    };

    match command {
        Commands::SelfUpdate { force } => cli::self_update::self_update(force).await?,
        Commands::Start { port } => cmd_start::cmd_start(port).await?,
        Commands::Plugin { command } => cmd_plugin::cmd_plugin(command).await?,
        Commands::Search { query } => cmd_search::cmd_search(&query).await?,
        Commands::Debug { command } => cmd_debug::cmd_debug(command).await?,
        Commands::Run { plugin_id, args } => cmd_run::cmd_run(plugin_id, args).await?,
        Commands::Completions { shell } => cmd_completions::cmd_completions(shell),
        Commands::Init { shell } => cmd_completions::cmd_init(shell)?,
        Commands::Logs {
            plugin_id,
            follow,
            lines,
            level,
            service,
        } => cmd_logs::cmd_logs(&plugin_id, follow, lines, level, service).await?,
        Commands::External(args) => cmd_external::cmd_external(args).await?,
    }

    Ok(())
}
