use adi_cli::completions::{self, CompletionShell};
use adi_cli::plugin_registry::PluginManager;
use adi_cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};

#[derive(Parser)]
#[command(name = "adi")]
#[command(version)]
#[command(about = "CLI for ADI family components", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update adi CLI itself to the latest version
    SelfUpdate {
        /// Force update even if already on latest version
        #[arg(long)]
        force: bool,
    },

    /// Manage plugins from the registry
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Search for plugins and packages in the registry
    Search {
        /// Search query
        query: String,
    },

    /// List registered services from loaded plugins
    Services,

    /// Run a plugin's CLI interface
    Run {
        /// Plugin ID to run (shows available plugins if omitted)
        plugin_id: Option<String>,

        /// Arguments to pass to the plugin
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: CompletionShell,
    },

    /// Initialize shell completions (writes to shell config)
    Init {
        /// Shell to initialize (auto-detects if not specified)
        #[arg(value_enum)]
        shell: Option<CompletionShell>,
    },

    /// Plugin-provided commands (dynamically discovered from installed plugins)
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Subcommand)]
enum PluginCommands {
    /// Search for plugins
    Search {
        /// Search query
        query: String,
    },

    /// List all available plugins
    List,

    /// List installed plugins
    Installed,

    /// Install a plugin or multiple plugins matching a pattern
    Install {
        /// Plugin ID (e.g., com.example.my-plugin) or pattern (e.g., adi.lang.*)
        plugin_id: String,

        /// Specific version to install
        #[arg(short, long)]
        version: Option<String>,
    },

    /// Update a plugin to latest version
    Update {
        /// Plugin ID
        plugin_id: String,
    },

    /// Update all installed plugins
    UpdateAll,

    /// Uninstall a plugin
    Uninstall {
        /// Plugin ID
        plugin_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Auto-initialize completions on first run
    completions::ensure_completions_installed::<Cli>("adi");

    let cli = Cli::parse();

    match cli.command {
        Commands::SelfUpdate { force } => adi_cli::self_update::self_update(force).await?,
        Commands::Plugin { command } => cmd_plugin(command).await?,
        Commands::Search { query } => cmd_search(&query).await?,
        Commands::Services => cmd_services().await?,
        Commands::Run { plugin_id, args } => cmd_run(plugin_id, args).await?,
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Init { shell } => cmd_init(shell)?,
        Commands::External(args) => cmd_external(args).await?,
    }

    Ok(())
}

fn cmd_completions(shell: CompletionShell) {
    completions::generate_completions::<Cli>(shell, "adi");
}

fn cmd_init(shell: Option<CompletionShell>) -> anyhow::Result<()> {
    let shell = shell
        .or_else(completions::detect_shell)
        .ok_or_else(|| anyhow::anyhow!(
            "Could not detect shell. Please specify: adi init bash|zsh|fish"
        ))?;

    println!(
        "{} shell completions for {}...",
        style("Initializing").bold(),
        style(format!("{:?}", shell)).cyan()
    );

    let path = completions::init_completions::<Cli>(shell, "adi")?;

    println!();
    println!(
        "{} Completions installed to: {}",
        style("Done!").green().bold(),
        style(path.display()).dim()
    );
    println!();

    match shell {
        CompletionShell::Zsh => {
            println!("Restart your shell or run:");
            println!("  {}", style("source ~/.zshrc").cyan());
        }
        CompletionShell::Bash => {
            println!("Restart your shell or run:");
            println!("  {}", style("source ~/.bashrc").cyan());
        }
        CompletionShell::Fish => {
            println!("Completions are active immediately in new fish sessions.");
        }
        _ => {
            println!("Restart your shell to enable completions.");
        }
    }

    Ok(())
}

async fn cmd_plugin(command: PluginCommands) -> anyhow::Result<()> {
    let manager = PluginManager::new();

    match command {
        PluginCommands::Search { query } => {
            cmd_search(&query).await?;
        }
        PluginCommands::List => {
            println!("{}", style("Available Plugins:").bold());
            println!();

            let plugins = manager.list_plugins().await?;

            if plugins.is_empty() {
                println!("  No plugins available in the registry.");
                return Ok(());
            }

            for plugin in plugins {
                println!(
                    "  {} {} - {} [{}]",
                    style(&plugin.id).cyan().bold(),
                    style(format!("v{}", plugin.latest_version)).dim(),
                    plugin.description,
                    style(&plugin.plugin_type).yellow()
                );
                if !plugin.tags.is_empty() {
                    println!("    Tags: {}", style(plugin.tags.join(", ")).dim());
                }
            }
        }
        PluginCommands::Installed => {
            println!("{}", style("Installed Plugins:").bold());
            println!();

            let installed = manager.list_installed().await?;

            if installed.is_empty() {
                println!("  No plugins installed.");
                println!();
                println!(
                    "  Install plugins with: {}",
                    style("adi plugin install <plugin-id>").cyan()
                );
                return Ok(());
            }

            for (id, version) in installed {
                println!(
                    "  {} {}",
                    style(&id).cyan().bold(),
                    style(format!("v{}", version)).dim(),
                );
            }
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
                println!("No plugins installed.");
                return Ok(());
            }

            println!(
                "{} {} plugin(s)...",
                style("Updating").bold(),
                installed.len()
            );

            for (id, _) in installed {
                if let Err(e) = manager.update_plugin(&id).await {
                    eprintln!(
                        "{} Failed to update {}: {}",
                        style("Warning:").yellow(),
                        id,
                        e
                    );
                }
            }

            println!();
            println!("{}", style("Update complete!").green().bold());
            regenerate_completions_quiet();
        }
        PluginCommands::Uninstall { plugin_id } => {
            let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Uninstall plugin {}?", plugin_id))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("Cancelled.");
                return Ok(());
            }

            manager.uninstall_plugin(&plugin_id).await?;
            regenerate_completions_quiet();
        }
    }

    Ok(())
}

/// Regenerate shell completions silently (called after plugin changes).
fn regenerate_completions_quiet() {
    if let Err(e) = completions::regenerate_completions::<Cli>("adi") {
        // Only warn in debug builds, silently ignore in release
        #[cfg(debug_assertions)]
        eprintln!(
            "{} Failed to regenerate completions: {}",
            style("Warning:").yellow(),
            e
        );
        let _ = e;
    }
}

async fn cmd_search(query: &str) -> anyhow::Result<()> {
    let manager = PluginManager::new();

    println!(
        "{} \"{}\"...",
        style("Searching for").bold(),
        style(query).cyan()
    );
    println!();

    let results = manager.search(query).await?;

    if results.packages.is_empty() && results.plugins.is_empty() {
        println!("  No results found.");
        return Ok(());
    }

    if !results.packages.is_empty() {
        println!("{}", style("Packages:").bold().underlined());
        for pkg in &results.packages {
            println!(
                "  {} {} - {}",
                style(&pkg.id).cyan().bold(),
                style(format!("v{}", pkg.latest_version)).dim(),
                pkg.description
            );
            if !pkg.tags.is_empty() {
                println!("    Tags: {}", style(pkg.tags.join(", ")).dim());
            }
        }
        println!();
    }

    if !results.plugins.is_empty() {
        println!("{}", style("Plugins:").bold().underlined());
        for plugin in &results.plugins {
            println!(
                "  {} {} - {} [{}]",
                style(&plugin.id).cyan().bold(),
                style(format!("v{}", plugin.latest_version)).dim(),
                plugin.description,
                style(&plugin.plugin_type).yellow()
            );
            if !plugin.tags.is_empty() {
                println!("    Tags: {}", style(plugin.tags.join(", ")).dim());
            }
        }
    }

    println!();
    println!(
        "Found {} package(s) and {} plugin(s)",
        results.packages.len(),
        results.plugins.len()
    );

    Ok(())
}

async fn cmd_services() -> anyhow::Result<()> {
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    let services = runtime.list_services();

    if services.is_empty() {
        println!("No services registered.");
        println!();
        println!(
            "Install plugins to add services: {}",
            style("adi plugin install <id>").cyan()
        );
        return Ok(());
    }

    println!("{}", style("Registered Services:").bold());
    println!();

    for service in services {
        println!(
            "  {} {} - {} [from {}]",
            style(service.id.as_str()).cyan().bold(),
            style(format!(
                "v{}.{}.{}",
                service.version.major, service.version.minor, service.version.patch
            ))
            .dim(),
            service.description.as_str(),
            style(service.provider_id.as_str()).yellow()
        );
    }

    Ok(())
}

async fn cmd_run(plugin_id: Option<String>, args: Vec<String>) -> anyhow::Result<()> {
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    // Get plugins with CLI services
    let runnable = runtime.list_runnable_plugins();

    // If no plugin_id provided, show available plugins
    let plugin_id = match plugin_id {
        Some(id) => id,
        None => {
            println!("{}", style("Runnable Plugins:").bold());
            println!();

            if runnable.is_empty() {
                println!("  No plugins with CLI interface installed.");
                println!();
                println!(
                    "  Install plugins with: {}",
                    style("adi plugin install <plugin-id>").cyan()
                );
            } else {
                for (id, description) in &runnable {
                    println!(
                        "  {} - {}",
                        style(id).cyan().bold(),
                        style(description).dim()
                    );
                }
                println!();
                println!(
                    "Run a plugin with: {}",
                    style("adi run <plugin-id> [args...]").cyan()
                );
            }
            return Ok(());
        }
    };

    // Check if plugin has CLI service
    if !runnable.iter().any(|(id, _)| id == &plugin_id) {
        eprintln!(
            "{} Plugin '{}' not found or has no CLI interface",
            style("Error:").red().bold(),
            plugin_id
        );
        eprintln!();
        if runnable.is_empty() {
            eprintln!("No runnable plugins installed.");
        } else {
            eprintln!("Runnable plugins:");
            for (id, _) in &runnable {
                eprintln!("  - {}", id);
            }
        }
        std::process::exit(1);
    }

    // Build CLI context
    let context = serde_json::json!({
        "command": plugin_id,
        "args": args,
        "cwd": std::env::current_dir()?.to_string_lossy()
    });

    match runtime.run_cli_command(&plugin_id, &context.to_string()) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{} Failed to run plugin: {}",
                style("Error:").red().bold(),
                e
            );
            std::process::exit(1);
        }
    }
}

/// Handle external (plugin-provided) commands.
///
/// This function discovers CLI commands from installed plugin manifests,
/// finds the matching plugin, loads it, and executes the command.
async fn cmd_external(args: Vec<String>) -> anyhow::Result<()> {
    if args.is_empty() {
        eprintln!("{} No command provided", style("Error:").red().bold());
        std::process::exit(1);
    }

    let command = args[0].clone();
    let cmd_args: Vec<String> = args.into_iter().skip(1).collect();

    // Create runtime to discover CLI commands from manifests
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

    // Discover CLI commands (fast - only reads manifests, no binary loading)
    let cli_commands = runtime.discover_cli_commands();

    // Find plugin by command name or alias
    let matching_plugin = cli_commands
        .iter()
        .find(|c| c.command == command || c.aliases.contains(&command));

    let Some(plugin_cmd) = matching_plugin else {
        // Command not found - show available commands
        eprintln!(
            "{} Unknown command: {}",
            style("Error:").red().bold(),
            &command
        );
        eprintln!();

        if cli_commands.is_empty() {
            eprintln!("No plugin commands installed.");
            eprintln!();
            eprintln!(
                "Install plugins with: {}",
                style("adi plugin install <plugin-id>").cyan()
            );
        } else {
            eprintln!("{}", style("Available plugin commands:").bold());
            for cmd in &cli_commands {
                let aliases = if cmd.aliases.is_empty() {
                    String::new()
                } else {
                    format!(" (aliases: {})", cmd.aliases.join(", "))
                };
                eprintln!(
                    "  {} - {}{}",
                    style(&cmd.command).cyan().bold(),
                    cmd.description,
                    style(aliases).dim()
                );
            }
        }
        std::process::exit(1);
    };

    let plugin_id = &plugin_cmd.plugin_id;

    // Now scan and load only the needed plugin
    if let Err(e) = runtime.scan_and_load_plugin(plugin_id).await {
        eprintln!(
            "{} Failed to load plugin '{}': {}",
            style("Error:").red().bold(),
            plugin_id,
            e
        );
        eprintln!();
        eprintln!(
            "Try reinstalling: {}",
            style(format!("adi plugin install {}", plugin_id)).cyan()
        );
        std::process::exit(1);
    }

    // Build CLI context
    let context = serde_json::json!({
        "command": plugin_id,
        "args": cmd_args,
        "cwd": std::env::current_dir()?.to_string_lossy()
    });

    match runtime.run_cli_command(plugin_id, &context.to_string()) {
        Ok(result) => {
            println!("{}", result);
            Ok(())
        }
        Err(e) => {
            eprintln!(
                "{} Failed to run {}: {}",
                style("Error:").red().bold(),
                command,
                e
            );
            std::process::exit(1);
        }
    }
}

