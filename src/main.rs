use adi_cli::completions::{self, CompletionShell};
use adi_cli::plugin_registry::PluginManager;
use adi_cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};
use lib_i18n_core::{init_global, t, I18n, ServiceRegistry as I18nServiceRegistry, ServiceDescriptor as I18nServiceDescriptor, ServiceHandle as I18nServiceHandle};
use lib_plugin_host::ServiceRegistry as PluginServiceRegistry;
use lib_plugin_abi::{ServiceError, ServiceHandle as PluginServiceHandle, ServiceDescriptor as PluginServiceDescriptor};
use std::sync::Arc;

#[derive(Parser)]
#[command(name = "adi")]
#[command(version)]
#[command(about = "CLI for ADI family components", long_about = None)]
struct Cli {
    /// Override language (e.g., en-US, zh-CN). Can also be set via ADI_LANG env var.
    #[arg(long, global = true)]
    lang: Option<String>,

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

    // Initialize i18n system
    initialize_i18n(cli.lang.as_deref()).await?;

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
        .ok_or_else(|| anyhow::anyhow!(t!("completions-error-no-shell")))?;

    println!(
        "{}",
        t!("completions-init-start", "shell" => &format!("{:?}", shell))
    );

    let path = completions::init_completions::<Cli>(shell, "adi")?;

    println!();
    println!(
        "{}",
        t!("completions-init-done", "path" => &path.display().to_string())
    );
    println!();

    match shell {
        CompletionShell::Zsh => {
            println!("{}", t!("completions-restart-zsh"));
        }
        CompletionShell::Bash => {
            println!("{}", t!("completions-restart-bash"));
        }
        CompletionShell::Fish => {
            println!("{}", t!("completions-restart-fish"));
        }
        _ => {
            println!("{}", t!("completions-restart-generic"));
        }
    }

    Ok(())
}

// Adapter to bridge lib-plugin-host ServiceRegistry to lib-i18n-core ServiceRegistry
struct ServiceRegistryAdapter {
    inner: Arc<PluginServiceRegistry>,
}

impl I18nServiceRegistry for ServiceRegistryAdapter {
    fn list_services(&self) -> lib_i18n_core::Result<Vec<I18nServiceDescriptor>> {
        Ok(self.inner.list()
            .into_iter()
            .map(|s: PluginServiceDescriptor| I18nServiceDescriptor::new(s.id.as_str().to_string()))
            .collect())
    }

    fn lookup_service(&self, service_id: &str) -> lib_i18n_core::Result<Box<dyn I18nServiceHandle>> {
        self.inner
            .lookup(service_id)
            .map(|handle| Box::new(ServiceHandleAdapter { inner: handle }) as Box<dyn I18nServiceHandle>)
            .ok_or_else(|| lib_i18n_core::I18nError::ServiceRegistryError(format!("Service not found: {}", service_id)))
    }
}

struct ServiceHandleAdapter {
    inner: PluginServiceHandle,
}

impl I18nServiceHandle for ServiceHandleAdapter {
    fn invoke(&self, method: &str, args: &str) -> lib_i18n_core::Result<String> {
        unsafe {
            self.inner
                .invoke(method, args)
                .map_err(|e: ServiceError| lib_i18n_core::I18nError::ServiceInvokeError(e.to_string()))
        }
    }
}

async fn initialize_i18n(lang_override: Option<&str>) -> anyhow::Result<()> {
    // Detect language from --lang flag, ADI_LANG env var, or LANG env var
    let user_lang = lang_override
        .map(|s| s.to_string())
        .or_else(|| std::env::var("ADI_LANG").ok())
        .or_else(|| {
            std::env::var("LANG")
                .ok()
                .and_then(|l| l.split('.').next().map(|s| s.replace('_', "-")))
        })
        .unwrap_or_else(|| "en-US".to_string());

    // Create plugin runtime and load translation plugin
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    let translation_id = format!("adi.cli.{}", user_lang);

    // Try user's language, fallback to English
    if runtime.scan_and_load_plugin(&translation_id).await.is_err() {
        let _ = runtime.scan_and_load_plugin("adi.cli.en-US").await;
    }

    // Initialize i18n with service registry (via adapter)
    let adapter = Arc::new(ServiceRegistryAdapter {
        inner: runtime.service_registry(),
    });
    let mut i18n = I18n::new(adapter).with_namespace("cli");
    i18n.discover_translations()?;
    i18n.set_language(&user_lang)?;
    init_global(i18n);

    Ok(())
}

async fn cmd_plugin(command: PluginCommands) -> anyhow::Result<()> {
    let manager = PluginManager::new();

    match command {
        PluginCommands::Search { query } => {
            cmd_search(&query).await?;
        }
        PluginCommands::List => {
            println!("{}", style(t!("plugin-list-title")).bold());
            println!();

            let plugins = manager.list_plugins().await?;

            if plugins.is_empty() {
                println!("  {}", t!("plugin-list-empty"));
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
            println!("{}", style(t!("plugin-installed-title")).bold());
            println!();

            let installed = manager.list_installed().await?;

            if installed.is_empty() {
                println!("  {}", t!("plugin-installed-empty"));
                println!();
                println!("  {}", t!("plugin-installed-hint"));
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
                println!("{}", t!("plugin-list-empty"));
                return Ok(());
            }

            println!(
                "{}",
                t!("plugin-update-all-start", "count" => &installed.len().to_string())
            );

            for (id, _) in installed {
                if let Err(e) = manager.update_plugin(&id).await {
                    eprintln!(
                        "{} {}",
                        style(t!("common-warning-prefix")).yellow(),
                        t!("plugin-update-all-warning", "id" => &id, "error" => &e.to_string())
                    );
                }
            }

            println!();
            println!("{}", style(t!("plugin-update-all-done")).green().bold());
            regenerate_completions_quiet();
        }
        PluginCommands::Uninstall { plugin_id } => {
            let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(t!("plugin-uninstall-prompt", "id" => &plugin_id))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{}", t!("plugin-uninstall-cancelled"));
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

    println!("{}", t!("search-searching", "query" => query));
    println!();

    let results = manager.search(query).await?;

    if results.packages.is_empty() && results.plugins.is_empty() {
        println!("  {}", t!("search-no-results"));
        return Ok(());
    }

    if !results.packages.is_empty() {
        println!("{}", style(t!("search-packages-title")).bold().underlined());
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
        println!("{}", style(t!("search-plugins-title")).bold().underlined());
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
        "{}",
        t!("search-results-summary",
            "packages" => &results.packages.len().to_string(),
            "plugins" => &results.plugins.len().to_string()
        )
    );

    Ok(())
}

async fn cmd_services() -> anyhow::Result<()> {
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    let services = runtime.list_services();

    if services.is_empty() {
        println!("{}", t!("services-empty"));
        println!();
        println!("{}", t!("services-hint"));
        return Ok(());
    }

    println!("{}", style(t!("services-title")).bold());
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
            println!("{}", style(t!("run-title")).bold());
            println!();

            if runnable.is_empty() {
                println!("  {}", t!("run-empty"));
                println!();
                println!("  {}", t!("run-hint-install"));
            } else {
                for (id, description) in &runnable {
                    println!(
                        "  {} - {}",
                        style(id).cyan().bold(),
                        style(description).dim()
                    );
                }
                println!();
                println!("{}", t!("run-hint-usage"));
            }
            return Ok(());
        }
    };

    // Check if plugin has CLI service
    if !runnable.iter().any(|(id, _)| id == &plugin_id) {
        eprintln!(
            "{} {}",
            style(t!("common-error-prefix")).red().bold(),
            t!("run-error-not-found", "id" => &plugin_id)
        );
        eprintln!();
        if runnable.is_empty() {
            eprintln!("{}", t!("run-error-no-plugins"));
        } else {
            eprintln!("{}", t!("run-error-available"));
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
                "{} {}",
                style(t!("common-error-prefix")).red().bold(),
                t!("run-error-failed", "error" => &e.to_string())
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
        eprintln!("{} {}", style(t!("common-error-prefix")).red().bold(), t!("external-error-no-command"));
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
            "{} {}",
            style(t!("common-error-prefix")).red().bold(),
            t!("external-error-unknown", "command" => &command)
        );
        eprintln!();

        if cli_commands.is_empty() {
            eprintln!("{}", t!("external-error-no-installed"));
            eprintln!();
            eprintln!("{}", t!("external-hint-install"));
        } else {
            eprintln!("{}", style(t!("external-available-title")).bold());
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
            "{} {}",
            style(t!("common-error-prefix")).red().bold(),
            t!("external-error-load-failed", "id" => plugin_id, "error" => &e.to_string())
        );
        eprintln!();
        eprintln!("{}", t!("external-hint-reinstall", "id" => plugin_id));
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
                "{} {}",
                style(t!("common-error-prefix")).red().bold(),
                t!("external-error-run-failed", "command" => &command, "error" => &e.to_string())
            );
            std::process::exit(1);
        }
    }
}

