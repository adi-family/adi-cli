mod args;
mod commands;
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

    match cli.command {
        Commands::SelfUpdate { force } => cli::self_update::self_update(force).await?,
        Commands::Start { port } => commands::start::cmd_start(port).await?,
        Commands::Plugin { command } => commands::plugin::cmd_plugin(command).await?,
        Commands::Search { query } => commands::search::cmd_search(&query).await?,
        Commands::Debug { command } => commands::debug::cmd_debug(command).await?,
        Commands::Run { plugin_id, args } => commands::run::cmd_run(plugin_id, args).await?,
        Commands::Completions { shell } => commands::completions::cmd_completions(shell),
        Commands::Init { shell } => commands::completions::cmd_init(shell)?,
        Commands::Logs {
            plugin_id,
            follow,
            lines,
            level,
            service,
        } => commands::logs::cmd_logs(&plugin_id, follow, lines, level, service).await?,
        Commands::External(args) => commands::external::cmd_external(args).await?,
    }

    Ok(())
}
