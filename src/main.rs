use adi_cli::completions::{self, CompletionShell};
use adi_cli::plugin_registry::PluginManager;
use adi_cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use adi_cli::user_config::UserConfig;
use clap::{Parser, Subcommand};
use console::style;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use lib_i18n_core::{
    init_global, t, I18n, ServiceDescriptor as I18nServiceDescriptor,
    ServiceHandle as I18nServiceHandle, ServiceRegistry as I18nServiceRegistry,
};
use lib_plugin_abi::{
    ServiceDescriptor as PluginServiceDescriptor, ServiceError,
    ServiceHandle as PluginServiceHandle,
};
use lib_plugin_host::ServiceRegistry as PluginServiceRegistry;
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

    /// Start local ADI server for browser connection
    Start {
        /// Port to listen on (default: 14730)
        #[arg(short, long, default_value = "14730")]
        port: u16,
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

    /// Debug and diagnostic commands
    Debug {
        #[command(subcommand)]
        command: DebugCommands,
    },

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

#[derive(Subcommand)]
enum DebugCommands {
    /// List registered services from loaded plugins
    Services,
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
        Commands::Start { port } => cmd_start(port).await?,
        Commands::Plugin { command } => cmd_plugin(command).await?,
        Commands::Search { query } => cmd_search(&query).await?,
        Commands::Debug { command } => cmd_debug(command).await?,
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
        Ok(self
            .inner
            .list()
            .into_iter()
            .map(|s: PluginServiceDescriptor| I18nServiceDescriptor::new(s.id.as_str().to_string()))
            .collect())
    }

    fn lookup_service(
        &self,
        service_id: &str,
    ) -> lib_i18n_core::Result<Box<dyn I18nServiceHandle>> {
        self.inner
            .lookup(service_id)
            .map(|handle| {
                Box::new(ServiceHandleAdapter { inner: handle }) as Box<dyn I18nServiceHandle>
            })
            .ok_or_else(|| {
                lib_i18n_core::I18nError::ServiceRegistryError(format!(
                    "Service not found: {}",
                    service_id
                ))
            })
    }
}

struct ServiceHandleAdapter {
    inner: PluginServiceHandle,
}

impl I18nServiceHandle for ServiceHandleAdapter {
    fn invoke(&self, method: &str, args: &str) -> lib_i18n_core::Result<String> {
        unsafe {
            self.inner.invoke(method, args).map_err(|e: ServiceError| {
                lib_i18n_core::I18nError::ServiceInvokeError(e.to_string())
            })
        }
    }
}

/// Available languages with display names
const AVAILABLE_LANGUAGES: &[(&str, &str)] = &[
    ("en-US", "English"),
    ("zh-CN", "中文 (简体)"),
    ("uk-UA", "Українська"),
    ("es-ES", "Español"),
    ("fr-FR", "Français"),
    ("de-DE", "Deutsch"),
    ("ja-JP", "日本語"),
    ("ko-KR", "한국어"),
];

/// Prompt user to select their preferred language interactively
fn prompt_language_selection() -> anyhow::Result<String> {
    println!();
    println!("{}", style("Welcome to ADI! 🎉").bold().cyan());
    println!();
    println!("Please select your preferred language:");
    println!();

    let items: Vec<String> = AVAILABLE_LANGUAGES
        .iter()
        .map(|(code, name)| format!("{} ({})", name, code))
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&items)
        .default(0)
        .interact()?;

    Ok(AVAILABLE_LANGUAGES[selection].0.to_string())
}

async fn initialize_i18n(lang_override: Option<&str>) -> anyhow::Result<()> {
    // Load or create user config
    let mut config = UserConfig::load()?;

    // Detect language with priority:
    // 1. CLI --lang flag (highest priority)
    // 2. ADI_LANG environment variable
    // 3. Saved user preference
    // 4. System LANG environment variable
    // 5. Interactive prompt on first run (if TTY)
    // 6. Default to en-US
    let user_lang = if let Some(lang) = lang_override {
        lang.to_string()
    } else if let Ok(env_lang) = std::env::var("ADI_LANG") {
        env_lang
    } else if let Some(saved_lang) = &config.language {
        saved_lang.clone()
    } else if let Ok(system_lang) = std::env::var("LANG") {
        system_lang
            .split('.')
            .next()
            .map(|s| s.replace('_', "-"))
            .unwrap_or_else(|| "en-US".to_string())
    } else if UserConfig::is_first_run()? && UserConfig::is_interactive() {
        // First run in interactive session - prompt user
        let selected_lang = prompt_language_selection()?;

        // Save the preference
        config.language = Some(selected_lang.clone());
        config.save()?;

        println!();
        println!(
            "{}",
            style(format!("Language set to: {}", selected_lang)).green()
        );
        println!("{}", style("You can change this later by setting ADI_LANG environment variable or using --lang flag").dim());
        println!();

        selected_lang
    } else {
        // Non-interactive or not first run - use default
        "en-US".to_string()
    };

    // Supported languages (must have translation plugins in registry)
    const SUPPORTED_LANGUAGES: &[&str] = &[
        "en-US", "de-DE", "es-ES", "fr-FR", "ja-JP", "ko-KR", "ru-RU", "uk-UA", "zh-CN",
    ];

    // Check if requested language is supported, fallback to en-US if not
    let effective_lang = if SUPPORTED_LANGUAGES.contains(&user_lang.as_str()) {
        user_lang.clone()
    } else {
        "en-US".to_string()
    };

    // Create plugin runtime and load translation plugin
    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    let translation_id = format!("adi.cli.{}", effective_lang);

    // Try to load user's language plugin
    if runtime.scan_and_load_plugin(&translation_id).await.is_err() {
        // Plugin not installed - try to install it automatically
        if effective_lang != "en-US" {
            println!(
                "{}",
                style(format!(
                    "Installing {} translation plugin...",
                    effective_lang
                ))
                .dim()
            );

            let manager = PluginManager::new();
            if let Ok(()) = manager.install_plugin(&translation_id, None).await {
                // Successfully installed, try loading again
                if runtime.scan_and_load_plugin(&translation_id).await.is_err() {
                    eprintln!("{}", style(format!("Warning: Failed to load {} after installation, falling back to English", translation_id)).yellow());
                    let _ = runtime.scan_and_load_plugin("adi.cli.en-US").await;
                }
            } else {
                // Installation failed, fallback to English
                eprintln!(
                    "{}",
                    style(format!(
                        "Warning: Translation plugin {} not available, using English",
                        translation_id
                    ))
                    .yellow()
                );
                let _ = runtime.scan_and_load_plugin("adi.cli.en-US").await;
            }
        } else {
            // English not found, try loading it anyway (shouldn't happen)
            let _ = runtime.scan_and_load_plugin("adi.cli.en-US").await;
        }
    }

    // Initialize i18n with service registry (via adapter)
    let adapter = Arc::new(ServiceRegistryAdapter {
        inner: runtime.service_registry(),
    });
    let mut i18n = I18n::new(adapter).with_namespace("cli");

    // Load embedded English translations as fallback (always available)
    let _ = i18n.load_embedded("en-US", include_str!("../plugins/en-US/messages.ftl"));

    // Discover additional translations from plugins
    i18n.discover_translations()?;

    // Try to set requested language, fallback to en-US if not available
    if i18n.set_language(&effective_lang).is_err() {
        let _ = i18n.set_language("en-US");
    }
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
                    let prefix = t!("common-warning-prefix");
                    let msg =
                        t!("plugin-update-all-warning", "id" => &id, "error" => &e.to_string());
                    eprintln!("{} {}", style(prefix).yellow(), msg);
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

async fn cmd_debug(command: DebugCommands) -> anyhow::Result<()> {
    match command {
        DebugCommands::Services => cmd_services().await?,
    }
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
        {
            let prefix = t!("common-error-prefix");
            let msg = t!("run-error-not-found", "id" => &plugin_id);
            eprintln!("{} {}", style(prefix).red().bold(), msg);
        }
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
            {
                let prefix = t!("common-error-prefix");
                let msg = t!("run-error-failed", "error" => &e.to_string());
                eprintln!("{} {}", style(prefix).red().bold(), msg);
            }
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
        {
            let prefix = t!("common-error-prefix");
            let msg = t!("external-error-no-command");
            eprintln!("{} {}", style(prefix).red().bold(), msg);
        }
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
        {
            let prefix = t!("common-error-prefix");
            let msg = t!("external-error-unknown", "command" => &command);
            eprintln!("{} {}", style(prefix).red().bold(), msg);
        }
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
        {
            let prefix = t!("common-error-prefix");
            let msg =
                t!("external-error-load-failed", "id" => plugin_id, "error" => &e.to_string());
            eprintln!("{} {}", style(prefix).red().bold(), msg);
        }
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
            {
                let prefix = t!("common-error-prefix");
                let msg = t!("external-error-run-failed", "command" => &command, "error" => &e.to_string());
                eprintln!("{} {}", style(prefix).red().bold(), msg);
            }
            std::process::exit(1);
        }
    }
}

/// Start local ADI server for browser connection.
///
/// This command:
/// 1. Ensures the cocoon plugin is installed
/// 2. Starts a local HTTP server with health + connect endpoints
/// 3. Waits for browser to send token via POST /connect
/// 4. Connects to signaling server with the token
async fn cmd_start(port: u16) -> anyhow::Result<()> {
    use axum::{routing::{get, post}, Router};
    use tower_http::cors::{Any, CorsLayer};
    use tokio::sync::RwLock;

    println!(
        "{}",
        style("Starting ADI local server...").cyan().bold()
    );

    // Ensure cocoon plugin is installed
    let manager = PluginManager::new();
    let installed = manager.list_installed().await?;
    let cocoon_installed = installed.iter().any(|(id, _)| id == "adi.cocoon");

    if !cocoon_installed {
        println!(
            "{}",
            style("Installing cocoon plugin...").dim()
        );
        manager.install_plugin("adi.cocoon", None).await?;
        println!(
            "{}",
            style("Cocoon plugin installed!").green()
        );
    }

    // Get machine hostname for display
    let hostname = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Local Machine".to_string());

    // Create channel for connection requests
    let (connect_tx, mut connect_rx) = tokio::sync::mpsc::channel::<ConnectRequest>(1);

    // Shared state for connection status
    let state = Arc::new(StartServerState {
        connected: RwLock::new(false),
        hostname: hostname.clone(),
        connect_tx,
    });

    // Create HTTP server with health and connect endpoints
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/connect", post(connect_handler))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    println!();
    println!(
        "  {} {}",
        style("Name:").dim(),
        style(&hostname).white().bold()
    );
    println!(
        "  {} {}",
        style("URL:").dim(),
        style(format!("http://localhost:{}", port)).cyan()
    );
    println!();
    println!(
        "{}",
        style("Waiting for browser connection... (Ctrl+C to stop)").dim()
    );
    println!();

    // Start HTTP server in background
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await
    });

    // Wait for connection request from browser
    if let Some(req) = connect_rx.recv().await {
        println!(
            "{}",
            style("Browser connected! Starting cocoon...").green().bold()
        );

        // Set environment variables for the cocoon plugin
        std::env::set_var("SIGNALING_SERVER_URL", &req.signaling_url);
        std::env::set_var("COCOON_SETUP_TOKEN", &req.token);

        // Load and run cocoon plugin
        let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

        runtime.scan_and_load_plugin("adi.cocoon").await?;

        // Run the cocoon (this will block and run the cocoon)
        let context = serde_json::json!({
            "command": "adi.cocoon",
            "args": ["run"],
            "cwd": std::env::current_dir().unwrap_or_default().to_string_lossy()
        });

        println!(
            "{}",
            style("Cocoon connected to platform!").green()
        );

        runtime.run_cli_command("adi.cocoon", &context.to_string())?;
    }

    // Abort the server task (cocoon finished or error)
    server.abort();

    Ok(())
}

/// Shared state for the start server
struct StartServerState {
    connected: tokio::sync::RwLock<bool>,
    hostname: String,
    /// Channel to send connection request to main loop
    connect_tx: tokio::sync::mpsc::Sender<ConnectRequest>,
}

/// Health endpoint handler for browser polling
async fn health_handler(
    axum::extract::State(state): axum::extract::State<Arc<StartServerState>>,
) -> axum::Json<serde_json::Value> {
    let connected = *state.connected.read().await;

    axum::Json(serde_json::json!({
        "status": "ok",
        "name": state.hostname,
        "version": env!("CARGO_PKG_VERSION"),
        "connected": connected
    }))
}

/// Request body for connect endpoint
#[derive(serde::Deserialize)]
struct ConnectRequest {
    token: String,
    #[serde(default = "default_signaling_url")]
    signaling_url: String,
}

fn default_signaling_url() -> String {
    "wss://adi.the-ihor.com/api/signaling/ws".to_string()
}

/// Connect endpoint - browser sends token to register with platform
async fn connect_handler(
    axum::extract::State(state): axum::extract::State<Arc<StartServerState>>,
    axum::Json(req): axum::Json<ConnectRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    // Check if already connected
    if *state.connected.read().await {
        return (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::json!({
                "status": "already_connected",
                "name": state.hostname
            })),
        );
    }

    // Send connection request to main loop
    if let Err(e) = state.connect_tx.send(req).await {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to process request: {}", e)
            })),
        );
    }

    // Mark as connected (cocoon will be started by main loop)
    *state.connected.write().await = true;

    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "status": "connecting",
            "name": state.hostname
        })),
    )
}
