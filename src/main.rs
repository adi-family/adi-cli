use adi_cli::component::{InstallConfig, InstallStatus};
use adi_cli::components::create_default_registry;
use adi_cli::installer::Installer;
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, MultiSelect};

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
    /// List all available components
    List,

    /// Install one or more components
    Install {
        /// Component names to install (interactive if not specified)
        #[arg(value_name = "COMPONENT")]
        components: Vec<String>,

        /// Install all components
        #[arg(long)]
        all: bool,
    },

    /// Uninstall one or more components
    Uninstall {
        /// Component names to uninstall
        #[arg(value_name = "COMPONENT")]
        components: Vec<String>,
    },

    /// Update one or more components
    Update {
        /// Component names to update (all installed if not specified)
        #[arg(value_name = "COMPONENT")]
        components: Vec<String>,
    },

    /// Show status of components
    Status {
        /// Component name (all if not specified)
        #[arg(value_name = "COMPONENT")]
        component: Option<String>,
    },

    /// Update adi CLI itself to the latest version
    SelfUpdate {
        /// Force update even if already on latest version
        #[arg(long)]
        force: bool,
    },

    /// Run ADI Code Indexer commands
    Indexer {
        #[command(subcommand)]
        command: Option<IndexerCommands>,

        /// Arguments to pass to indexer CLI
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum IndexerCommands {
    /// Run MCP server
    Mcp {
        #[command(subcommand)]
        command: Option<McpCommands>,

        /// Arguments to pass to MCP server (when no subcommand is used)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Run HTTP server
    Http {
        /// Arguments to pass to HTTP server
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum McpCommands {
    /// Initialize MCP server integration with external tools
    Init {
        /// Target tool to integrate with (e.g., "claude-code")
        target: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let registry = create_default_registry();
    let installer = Installer::new(registry);
    let config = InstallConfig::default();

    match cli.command {
        Commands::List => cmd_list(&installer).await?,
        Commands::Install { components, all } => {
            cmd_install(&installer, components, all, &config).await?
        }
        Commands::Uninstall { components } => cmd_uninstall(&installer, components).await?,
        Commands::Update { components } => cmd_update(&installer, components, &config).await?,
        Commands::Status { component } => cmd_status(&installer, component).await?,
        Commands::SelfUpdate { force } => adi_cli::self_update::self_update(force).await?,
        Commands::Indexer { command, args } => cmd_indexer(command, args).await?,
    }

    Ok(())
}

async fn cmd_list(installer: &Installer) -> anyhow::Result<()> {
    println!("{}", style("Available ADI Components:").bold());
    println!();

    for component in installer.registry().list() {
        let info = component.info();
        let status = component.status().await?;

        let status_str = match status {
            InstallStatus::NotInstalled => style("not installed").dim(),
            InstallStatus::Installed => style("installed").green(),
            InstallStatus::UpdateAvailable => style("update available").yellow(),
        };

        println!(
            "  {} {} - {} [{}]",
            style(&info.name).cyan().bold(),
            style(format!("v{}", info.version)).dim(),
            info.description,
            status_str
        );

        if !info.dependencies.is_empty() {
            println!(
                "    Dependencies: {}",
                style(info.dependencies.join(", ")).dim()
            );
        }
    }

    Ok(())
}

async fn cmd_install(
    installer: &Installer,
    components: Vec<String>,
    all: bool,
    config: &InstallConfig,
) -> anyhow::Result<()> {
    let to_install = if all {
        installer
            .registry()
            .names()
            .into_iter()
            .map(String::from)
            .collect()
    } else if components.is_empty() {
        interactive_select_components(installer, "Select components to install:").await?
    } else {
        components
    };

    if to_install.is_empty() {
        println!("No components selected.");
        return Ok(());
    }

    println!("{}", style("Installing components...").bold());

    for name in to_install {
        installer.install(&name, config).await?;
    }

    println!();
    println!("{}", style("Installation complete!").green().bold());

    Ok(())
}

async fn cmd_uninstall(installer: &Installer, components: Vec<String>) -> anyhow::Result<()> {
    let to_uninstall = if components.is_empty() {
        interactive_select_components(installer, "Select components to uninstall:").await?
    } else {
        components
    };

    if to_uninstall.is_empty() {
        println!("No components selected.");
        return Ok(());
    }

    let confirmed = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Uninstall {} component(s)?", to_uninstall.len()))
        .default(false)
        .interact()?;

    if !confirmed {
        println!("Cancelled.");
        return Ok(());
    }

    for name in to_uninstall {
        installer.uninstall(&name).await?;
    }

    println!();
    println!("{}", style("Uninstallation complete!").green().bold());

    Ok(())
}

async fn cmd_update(
    installer: &Installer,
    components: Vec<String>,
    config: &InstallConfig,
) -> anyhow::Result<()> {
    let to_update = if components.is_empty() {
        let mut installed = Vec::new();
        for component in installer.registry().list() {
            if component.status().await? == InstallStatus::Installed {
                installed.push(component.info().name.clone());
            }
        }
        installed
    } else {
        components
    };

    if to_update.is_empty() {
        println!("No components to update.");
        return Ok(());
    }

    println!("{}", style("Updating components...").bold());

    for name in to_update {
        installer.update(&name, config).await?;
    }

    println!();
    println!("{}", style("Update complete!").green().bold());

    Ok(())
}

async fn cmd_status(installer: &Installer, component: Option<String>) -> anyhow::Result<()> {
    match component {
        Some(name) => {
            let status = installer.status(&name).await?;
            let status_str = match status {
                InstallStatus::NotInstalled => "not installed",
                InstallStatus::Installed => "installed",
                InstallStatus::UpdateAvailable => "update available",
            };
            println!("{}: {}", name, status_str);
        }
        None => {
            cmd_list(installer).await?;
        }
    }

    Ok(())
}

async fn interactive_select_components(
    installer: &Installer,
    prompt: &str,
) -> anyhow::Result<Vec<String>> {
    let components: Vec<_> = installer.registry().list();
    let items: Vec<String> = components
        .iter()
        .map(|c| {
            let info = c.info();
            format!("{} - {}", info.name, info.description)
        })
        .collect();

    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .interact()?;

    let selected_names = selections
        .into_iter()
        .map(|i| components[i].info().name.clone())
        .collect();

    Ok(selected_names)
}

async fn cmd_indexer(command: Option<IndexerCommands>, args: Vec<String>) -> anyhow::Result<()> {
    let bin_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("adi")
        .join("bin");

    match command {
        Some(IndexerCommands::Mcp {
            command: mcp_cmd,
            args: mcp_args,
        }) => {
            match mcp_cmd {
                Some(McpCommands::Init { target }) => {
                    cmd_mcp_init(&target, &bin_dir).await?;
                }
                None => {
                    // Delegate to binary
                    let binary_path = bin_dir.join("adi-indexer-mcp");
                    run_binary(&binary_path, "indexer-mcp", &mcp_args)?;
                }
            }
        }
        Some(IndexerCommands::Http { args: http_args }) => {
            let binary_path = bin_dir.join("adi-indexer-http");
            run_binary(&binary_path, "llm-code-indexer-http", &http_args)?;
        }
        None => {
            let binary_path = bin_dir.join("adi-indexer-cli");
            run_binary(&binary_path, "indexer-cli", &args)?;
        }
    }

    Ok(())
}

fn run_binary(
    binary_path: &std::path::Path,
    component_name: &str,
    args: &[String],
) -> anyhow::Result<()> {
    use std::process::Command;

    if !binary_path.exists() {
        eprintln!(
            "{} Binary not found: {}",
            style("Error:").red().bold(),
            binary_path.display()
        );
        eprintln!(
            "Install it with: {}",
            style(format!("adi install {}", component_name)).cyan()
        );
        std::process::exit(1);
    }

    let status = Command::new(binary_path).args(args).status()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

async fn cmd_mcp_init(target: &str, bin_dir: &std::path::Path) -> anyhow::Result<()> {
    use std::process::Command;

    match target {
        "claude-code" => {
            let binary_path = bin_dir.join("adi-indexer-mcp");

            if !binary_path.exists() {
                eprintln!(
                    "{} MCP server binary not found: {}",
                    style("Error:").red().bold(),
                    binary_path.display()
                );
                eprintln!(
                    "Install it first with: {}",
                    style("adi install indexer-mcp").cyan()
                );
                std::process::exit(1);
            }

            println!(
                "{}",
                style("Registering MCP server with Claude Code...").bold()
            );

            let status = Command::new("claude")
                .args([
                    "mcp",
                    "add",
                    "--transport",
                    "stdio",
                    "indexer-mcp",
                    "--",
                    binary_path.to_str().unwrap(),
                ])
                .status()?;

            if !status.success() {
                eprintln!(
                    "{} Failed to register MCP server with Claude Code",
                    style("Error:").red().bold()
                );
                eprintln!("Make sure you have Claude Code CLI installed.");
                std::process::exit(status.code().unwrap_or(1));
            }

            println!(
                "{}",
                style("✓ MCP server registered successfully!")
                    .green()
                    .bold()
            );
            println!();
            println!("You can now use the indexer-mcp server in Claude Code:");
            println!("  • Reference resources: @indexer-mcp/resource-name");
            println!("  • Check status: /mcp");
            println!("  • View registered servers: claude mcp list");
        }
        _ => {
            eprintln!(
                "{} Unknown target: {}",
                style("Error:").red().bold(),
                target
            );
            eprintln!("Supported targets: claude-code");
            std::process::exit(1);
        }
    }

    Ok(())
}
