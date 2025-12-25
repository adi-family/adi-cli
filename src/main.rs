use std::sync::Arc;

use adi_cli::http_server::{HttpServer, HttpServerConfig};
use adi_cli::mcp_server::McpServer;
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

    /// Start MCP server (JSON-RPC over stdio)
    Mcp,

    /// Start HTTP server for plugin-provided routes
    Http {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
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

    /// Task management - dependency graphs and task tracking
    Tasks {
        /// Arguments to pass to task manager
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Agent loop - autonomous LLM agents for code tasks
    #[command(name = "agent-loop")]
    AgentLoop {
        /// Arguments to pass to agent loop
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
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
    let cli = Cli::parse();

    match cli.command {
        Commands::SelfUpdate { force } => adi_cli::self_update::self_update(force).await?,
        Commands::Plugin { command } => cmd_plugin(command).await?,
        Commands::Search { query } => cmd_search(&query).await?,
        Commands::Mcp => cmd_mcp().await?,
        Commands::Http { port, host } => cmd_http(port, host).await?,
        Commands::Services => cmd_services().await?,
        Commands::Run { plugin_id, args } => cmd_run(plugin_id, args).await?,
        Commands::Tasks { args } => cmd_plugin_direct("adi.tasks", args).await?,
        Commands::AgentLoop { args } => cmd_plugin_direct("adi.agent-loop", args).await?,
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
        }
        PluginCommands::Update { plugin_id } => {
            manager.update_plugin(&plugin_id).await?;
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
        }
    }

    Ok(())
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

async fn cmd_mcp() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    // Create plugin runtime and load plugins
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    // Create and run MCP server
    #[allow(clippy::arc_with_non_send_sync)]
    let runtime_arc = Arc::new(runtime);
    let mut server = McpServer::new(runtime_arc);
    server.run().await?;

    Ok(())
}

async fn cmd_http(port: u16, host: String) -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let config = HttpServerConfig {
        port,
        host: host.clone(),
    };

    println!(
        "{}",
        style(format!("Starting HTTP server on {}:{}", host, port)).bold()
    );

    let server = HttpServer::new(config);
    server.run().await?;

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

async fn cmd_plugin_direct(plugin_id: &str, args: Vec<String>) -> anyhow::Result<()> {
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    // Check if plugin has CLI service
    let service_id = format!("{}.cli", plugin_id);
    if !runtime.has_service(&service_id) {
        eprintln!(
            "{} Plugin '{}' not found or not installed",
            style("Error:").red().bold(),
            plugin_id
        );
        eprintln!();
        eprintln!(
            "Install with: {}",
            style(format!("adi plugin install {}", plugin_id)).cyan()
        );
        std::process::exit(1);
    }

    // Build CLI context
    let context = serde_json::json!({
        "command": plugin_id,
        "args": args,
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
                plugin_id,
                e
            );
            std::process::exit(1);
        }
    }
}
