use cli::completions::{self, CompletionShell};
use cli::plugin_registry::PluginManager;
use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use cli::user_config::UserConfig;
use clap::{Parser, Subcommand};
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use lib_console_output::theme;
use lib_i18n_core::{init_global, t, I18n};
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

    /// Stream live logs from a plugin
    Logs {
        /// Plugin ID to stream logs from (e.g., adi.hive)
        plugin_id: String,

        /// Follow log output (stream continuously)
        #[arg(short = 'f', long)]
        follow: bool,

        /// Number of recent lines to show
        #[arg(short = 'n', long, default_value = "50")]
        lines: u32,

        /// Minimum log level (trace, debug, info, warn, error, fatal)
        #[arg(long)]
        level: Option<String>,

        /// Filter by service name
        #[arg(long)]
        service: Option<String>,
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

    /// Show installation path for a plugin
    Path {
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
    // Load .env file from current directory (silently ignore if missing)
    dotenvy::dotenv().ok();

    // Auto-initialize completions on first run
    completions::ensure_completions_installed::<Cli>("adi");

    let cli = Cli::parse();

    // Initialize i18n system
    initialize_i18n(cli.lang.as_deref()).await?;

    // Initialize theme: ADI_THEME env var > config file > default
    initialize_theme();

    match cli.command {
        Commands::SelfUpdate { force } => cli::self_update::self_update(force).await?,
        Commands::Start { port } => cmd_start(port).await?,
        Commands::Plugin { command } => cmd_plugin(command).await?,
        Commands::Search { query } => cmd_search(&query).await?,
        Commands::Debug { command } => cmd_debug(command).await?,
        Commands::Run { plugin_id, args } => cmd_run(plugin_id, args).await?,
        Commands::Completions { shell } => cmd_completions(shell),
        Commands::Init { shell } => cmd_init(shell)?,
        Commands::Logs {
            plugin_id,
            follow,
            lines,
            level,
            service,
        } => cmd_logs(&plugin_id, follow, lines, level, service).await?,
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



/// Initialize the ADI theme from env var or user config.
///
/// Priority: ADI_THEME env var > config file theme > default ("indigo").
fn initialize_theme() {
    let theme_id = cli::clienv::theme()
        .or_else(|| UserConfig::load().ok().and_then(|c| c.theme))
        .unwrap_or_else(|| lib_console_output::theme::generated::DEFAULT_THEME.to_string());
    lib_console_output::theme::init(&theme_id);
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
    println!("{}", theme::brand_bold("Welcome to ADI! 🎉"));
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
    } else if let Some(env_lang) = cli::clienv::lang() {
        env_lang
    } else if let Some(saved_lang) = &config.language {
        saved_lang.clone()
    } else if let Some(system_lang) = cli::clienv::system_lang() {
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
            theme::success(format!("Language set to: {}", selected_lang))
        );
        println!("{}", theme::muted("You can change this later by setting ADI_LANG environment variable or using --lang flag"));
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

    // Initialize i18n with direct FTL file loading (no plugin service registry needed)
    let mut i18n = I18n::new_standalone();

    // Load embedded English translations as fallback (always available)
    let _ = i18n.load_embedded("en-US", include_str!("../plugins/en-US/messages.ftl"));

    // Try to load additional language from installed plugins
    if effective_lang != "en-US" {
        let translation_id = format!("adi.cli.{}", effective_lang);
        
        // Get plugins directory
        let plugins_dir = dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("adi")
            .join("plugins");

        // Try to load from installed plugin
        let plugin_dir = plugins_dir.join(&translation_id);
        let ftl_loaded = if let Some(ftl_path) = find_messages_ftl(&plugin_dir) {
            if let Ok(ftl_content) = std::fs::read_to_string(&ftl_path) {
                i18n.load_embedded(&effective_lang, &ftl_content).is_ok()
            } else {
                false
            }
        } else {
            false
        };

        // If not found, try to install from registry
        if !ftl_loaded {
            println!(
                "{}",
                theme::muted(format!(
                    "Installing {} translation plugin...",
                    effective_lang
                ))
            );

            let manager = PluginManager::new();
            if manager.install_plugin(&translation_id, None).await.is_ok() {
                // Successfully installed, try loading again
                if let Some(ftl_path) = find_messages_ftl(&plugin_dir) {
                    if let Ok(ftl_content) = std::fs::read_to_string(&ftl_path) {
                        let _ = i18n.load_embedded(&effective_lang, &ftl_content);
                    }
                }
            } else {
                eprintln!(
                    "{}",
                    theme::warning(format!(
                        "Warning: Translation plugin {} not available, using English",
                        translation_id
                    ))
                );
            }
        }
    }

    // Try to set requested language, fallback to en-US if not available
    if i18n.set_language(&effective_lang).is_err() {
        let _ = i18n.set_language("en-US");
    }
    init_global(i18n);

    Ok(())
}

/// Find the messages.ftl file in a plugin directory (handles versioned directories)
fn find_messages_ftl(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    // Check for .version file to get current version
    let version_file = plugin_dir.join(".version");
    if version_file.exists() {
        if let Ok(version) = std::fs::read_to_string(&version_file) {
            let version = version.trim();
            let ftl_path = plugin_dir.join(version).join("messages.ftl");
            if ftl_path.exists() {
                return Some(ftl_path);
            }
        }
    }

    // Fallback: check for messages.ftl directly in plugin dir
    let direct_ftl = plugin_dir.join("messages.ftl");
    if direct_ftl.exists() {
        return Some(direct_ftl);
    }

    // Fallback: scan subdirectories for messages.ftl
    if let Ok(entries) = std::fs::read_dir(plugin_dir) {
        for entry in entries.flatten() {
            let subdir = entry.path();
            if subdir.is_dir() {
                let ftl_path = subdir.join("messages.ftl");
                if ftl_path.exists() {
                    return Some(ftl_path);
                }
            }
        }
    }

    None
}

async fn cmd_plugin(command: PluginCommands) -> anyhow::Result<()> {
    let manager = PluginManager::new();

    match command {
        PluginCommands::Search { query } => {
            cmd_search(&query).await?;
        }
        PluginCommands::List => {
            println!("{}", theme::bold(t!("plugin-list-title")));
            println!();

            let plugins = manager.list_plugins().await?;

            if plugins.is_empty() {
                println!("  {}", t!("plugin-list-empty"));
                return Ok(());
            }

            for plugin in plugins {
                println!(
                    "  {} {} - {} [{}]",
                    theme::brand_bold(&plugin.id),
                    theme::muted(format!("v{}", plugin.latest_version)),
                    plugin.description,
                    theme::warning(&plugin.plugin_type)
                );
                if !plugin.tags.is_empty() {
                    println!("    Tags: {}", theme::muted(plugin.tags.join(", ")));
                }
            }
        }
        PluginCommands::Installed => {
            println!("{}", theme::bold(t!("plugin-installed-title")));
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
                    theme::brand_bold(&id),
                    theme::muted(format!("v{}", version)),
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
                    eprintln!("{} {}", theme::warning(prefix), msg);
                }
            }

            println!();
            println!("{}", theme::success(t!("plugin-update-all-done")));
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
        PluginCommands::Path { plugin_id } => {
            let plugin_dir = manager.plugin_path(&plugin_id);
            let version_file = plugin_dir.join(".version");

            if !version_file.exists() {
                eprintln!(
                    "{} Plugin {} is not installed",
                    theme::error("Error:"),
                    theme::brand(&plugin_id)
                );
                std::process::exit(1);
            }

            let version = tokio::fs::read_to_string(&version_file).await?;
            let version = version.trim();
            let versioned_path = plugin_dir.join(version);

            // Print just the path (useful for scripting)
            println!("{}", versioned_path.display());
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
            theme::warning("Warning:"),
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
        println!("{}", theme::bold(t!("search-packages-title")));
        for pkg in &results.packages {
            println!(
                "  {} {} - {}",
                theme::brand_bold(&pkg.id),
                theme::muted(format!("v{}", pkg.latest_version)),
                pkg.description
            );
            if !pkg.tags.is_empty() {
                println!("    Tags: {}", theme::muted(pkg.tags.join(", ")));
            }
        }
        println!();
    }

    if !results.plugins.is_empty() {
        println!("{}", theme::bold(t!("search-plugins-title")));
        for plugin in &results.plugins {
            println!(
                "  {} {} - {} [{}]",
                theme::brand_bold(&plugin.id),
                theme::muted(format!("v{}", plugin.latest_version)),
                plugin.description,
                theme::warning(&plugin.plugin_type)
            );
            if !plugin.tags.is_empty() {
                println!("    Tags: {}", theme::muted(plugin.tags.join(", ")));
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

    let plugins = runtime.list_installed();

    if plugins.is_empty() {
        println!("{}", t!("services-empty"));
        println!();
        println!("{}", t!("services-hint"));
        return Ok(());
    }

    println!("{}", theme::bold(t!("services-title")));
    println!();

    for plugin_id in plugins {
        println!("  {}", theme::brand_bold(&plugin_id));
    }

    Ok(())
}

/// Handle CLI result from plugin execution.
/// Parses the JSON result and outputs stdout/stderr appropriately,
/// then exits with the correct exit code.
fn handle_cli_result(result_json: &str) {
    #[derive(serde::Deserialize)]
    struct CliResult {
        exit_code: i32,
        stdout: String,
        stderr: String,
    }

    match serde_json::from_str::<CliResult>(result_json) {
        Ok(result) => {
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            if result.exit_code != 0 {
                std::process::exit(result.exit_code);
            }
        }
        Err(_) => {
            // Fallback: if JSON parsing fails, print raw result
            println!("{}", result_json);
        }
    }
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
            println!("{}", theme::bold(t!("run-title")));
            println!();

            if runnable.is_empty() {
                println!("  {}", t!("run-empty"));
                println!();
                println!("  {}", t!("run-hint-install"));
            } else {
                for (id, description) in &runnable {
                    println!(
                        "  {} - {}",
                        theme::brand_bold(id),
                        theme::muted(description)
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
            eprintln!("{} {}", theme::error(prefix), msg);
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

    match runtime.run_cli_command(&plugin_id, &context.to_string()).await {
        Ok(result) => {
            handle_cli_result(&result);
            Ok(())
        }
        Err(e) => {
            {
                let prefix = t!("common-error-prefix");
                let msg = t!("run-error-failed", "error" => &e.to_string());
                eprintln!("{} {}", theme::error(prefix), msg);
            }
            std::process::exit(1);
        }
    }
}

/// Stream logs from a plugin's LogProvider.
async fn cmd_logs(
    plugin_id: &str,
    follow: bool,
    lines: u32,
    level: Option<String>,
    service: Option<String>,
) -> anyhow::Result<()> {
    use lib_plugin_abi_v3::logs::LogStreamContext;

    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

    // Load the target plugin
    if let Err(e) = runtime.scan_and_load_plugin(plugin_id).await {
        eprintln!(
            "{} Failed to load plugin {}: {}",
            theme::error("Error:"),
            plugin_id,
            e
        );
        std::process::exit(1);
    }

    // Get the log provider
    let log_provider = runtime.get_log_provider(plugin_id);
    let log_provider = match log_provider {
        Some(p) => p,
        None => {
            eprintln!(
                "{} Plugin {} does not provide log streaming.",
                theme::error("Error:"),
                theme::brand(plugin_id)
            );
            std::process::exit(1);
        }
    };

    let ctx = LogStreamContext {
        service,
        level,
        tail: Some(lines),
        follow,
    };

    let mut stream = log_provider.log_stream(ctx).await.map_err(|e| {
        anyhow::anyhow!("Failed to create log stream: {}", e)
    })?;

    while let Some(line) = stream.next().await {
        let level_colored = match line.level.as_str() {
            "trace" => theme::muted(&line.level).to_string(),
            "debug" => theme::debug(&line.level).to_string(),
            "info" => theme::success(&line.level).to_string(),
            "notice" => theme::debug(&line.level).to_string(),
            "warn" => theme::warning(&line.level).to_string(),
            "error" => theme::error(&line.level).to_string(),
            "fatal" => theme::brand_bold(&line.level).to_string(),
            _ => line.level.clone(),
        };
        let timestamp = line.timestamp.format("%H:%M:%S%.3f");
        println!("{} {} [{}] {}", timestamp, line.service, level_colored, line.message);
    }

    Ok(())
}

/// Handle external (plugin-provided) commands.
///
/// This function discovers CLI commands from installed plugin manifests,
/// finds the matching plugin, loads it, and executes the command.
///
/// If the command is not found in installed plugins, it will attempt to
/// auto-install the plugin from the registry (following the `adi.{command}` pattern).
async fn cmd_external(args: Vec<String>) -> anyhow::Result<()> {
    if args.is_empty() {
        {
            let prefix = t!("common-error-prefix");
            let msg = t!("external-error-no-command");
            eprintln!("{} {}", theme::error(prefix), msg);
        }
        std::process::exit(1);
    }

    let command = args[0].clone();
    let cmd_args: Vec<String> = args.into_iter().skip(1).collect();

    // Create runtime to discover CLI commands from manifests
    let mut runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

    // Discover CLI commands (fast - only reads manifests, no binary loading)
    let cli_commands = runtime.discover_cli_commands();

    // Find plugin by command name or alias
    let matching_plugin = cli_commands
        .iter()
        .find(|c| c.command == command || c.aliases.contains(&command));

    let plugin_id = if let Some(plugin_cmd) = matching_plugin {
        plugin_cmd.plugin_id.clone()
    } else {
        // Command not found in installed plugins - try auto-install
        match try_autoinstall_plugin(&command, &cli_commands).await {
            AutoinstallResult::Installed(id) => {
                // Refresh the runtime to pick up the newly installed plugin
                runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
                id
            }
            AutoinstallResult::NotFound => {
                std::process::exit(1);
            }
            AutoinstallResult::Declined => {
                std::process::exit(1);
            }
            AutoinstallResult::Failed => {
                std::process::exit(1);
            }
        }
    };

    // Now scan and load only the needed plugin
    if let Err(e) = runtime.scan_and_load_plugin(&plugin_id).await {
        {
            let prefix = t!("common-error-prefix");
            let msg =
                t!("external-error-load-failed", "id" => &plugin_id, "error" => &e.to_string());
            eprintln!("{} {}", theme::error(prefix), msg);
        }
        eprintln!();
        eprintln!("{}", t!("external-hint-reinstall", "id" => &plugin_id));
        std::process::exit(1);
    }

    // Build CLI context
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
            {
                let prefix = t!("common-error-prefix");
                let msg = t!("external-error-run-failed", "command" => &command, "error" => &e.to_string());
                eprintln!("{} {}", theme::error(prefix), msg);
            }
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
/// For example: `hive` -> `adi.cli.hive`, `agent-loop` -> `adi.cli.agent-loop`.
///
/// Note: Core plugins use `adi.{name}` pattern (e.g., `adi.hive`), but CLI plugins
/// that provide the `adi <command>` interface use `adi.cli.{name}` pattern.
async fn try_autoinstall_plugin(
    command: &str,
    cli_commands: &[cli::plugin_runtime::PluginCliCommand],
) -> AutoinstallResult {
    use std::io::{self, Write};

    // Check if auto-install is disabled
    let auto_install_disabled = cli::clienv::auto_install_disabled();

    // Infer plugin ID from command: hive -> adi.cli.hive
    let plugin_id = format!("adi.cli.{}", command);

    // Try to check if the plugin exists in the registry
    let manager = PluginManager::new();

    // Try to get plugin info from registry
    match manager.get_plugin_info(&plugin_id).await {
        Ok(Some(_info)) => {
            // Plugin found in registry
            println!(
                "{}",
                t!("external-autoinstall-found", "id" => &plugin_id, "command" => command)
            );

            if auto_install_disabled {
                eprintln!("{}", t!("external-autoinstall-disabled", "id" => &plugin_id));
                return AutoinstallResult::Declined;
            }

            // Check if running in non-interactive mode (no TTY)
            let is_interactive = atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout);

            let should_install = if is_interactive {
                // Prompt user for confirmation
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
                // Non-interactive: auto-install by default
                true
            };

            if !should_install {
                eprintln!("{}", t!("external-autoinstall-disabled", "id" => &plugin_id));
                return AutoinstallResult::Declined;
            }

            // Install the plugin
            println!("{}", t!("external-autoinstall-installing", "id" => &plugin_id));

            match manager.install_with_dependencies(&plugin_id, None).await {
                Ok(()) => {
                    println!(
                        "{} {}",
                        theme::success(t!("common-success-prefix")),
                        t!("external-autoinstall-success")
                    );
                    eprintln!(); // Blank line before command output
                    AutoinstallResult::Installed(plugin_id)
                }
                Err(e) => {
                    eprintln!(
                        "{} {}",
                        theme::error(t!("common-error-prefix")),
                        t!("external-autoinstall-failed", "error" => &e.to_string())
                    );
                    AutoinstallResult::Failed
                }
            }
        }
        Ok(None) | Err(_) => {
            // Plugin not found in registry - show standard error
            {
                let prefix = t!("common-error-prefix");
                let msg = t!("external-error-unknown", "command" => command);
                eprintln!("{} {}", theme::error(prefix), msg);
            }
            eprintln!();

            // Show hint about registry plugin pattern
            eprintln!("{}", t!("external-autoinstall-not-found", "command" => command));
            eprintln!();

            if cli_commands.is_empty() {
                eprintln!("{}", t!("external-error-no-installed"));
                eprintln!();
                eprintln!("{}", t!("external-hint-install"));
            } else {
                eprintln!("{}", theme::bold(t!("external-available-title")));
                for cmd in cli_commands {
                    let aliases = if cmd.aliases.is_empty() {
                        String::new()
                    } else {
                        format!(" (aliases: {})", cmd.aliases.join(", "))
                    };
                    eprintln!(
                        "  {} - {}{}",
                        theme::brand_bold(&cmd.command),
                        cmd.description,
                        theme::muted(aliases)
                    );
                }
            }

            AutoinstallResult::NotFound
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
        theme::brand_bold("Starting ADI local server...")
    );

    // Ensure cocoon plugin is installed
    let manager = PluginManager::new();
    let installed = manager.list_installed().await?;
    let cocoon_installed = installed.iter().any(|(id, _)| id == "adi.cocoon");

    if !cocoon_installed {
        println!(
            "{}",
            theme::muted("Installing cocoon plugin...")
        );
        manager.install_plugin("adi.cocoon", None).await?;
        println!(
            "{}",
            theme::success("Cocoon plugin installed!")
        );
    }

    // Get machine name for display
    let hostname = get_machine_name();

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

    // Detect capabilities
    let capabilities = detect_capabilities();
    let ai_agents: Vec<_> = capabilities.iter()
        .filter(|c| c.category == "ai-agent")
        .map(|c| c.name)
        .collect();
    let runtimes: Vec<_> = capabilities.iter()
        .filter(|c| c.category == "runtime")
        .map(|c| c.name)
        .collect();

    println!();
    println!(
        "  {} {}",
        theme::muted("Name:"),
        theme::bold(&hostname)
    );
    println!(
        "  {} {}",
        theme::muted("URL:"),
        theme::brand(format!("http://localhost:{}", port))
    );
    if !ai_agents.is_empty() {
        println!(
            "  {} {}",
            theme::muted("Agents:"),
            theme::success(ai_agents.join(", "))
        );
    }
    if !runtimes.is_empty() {
        println!(
            "  {} {}",
            theme::muted("Runtimes:"),
            theme::info(runtimes.join(", "))
        );
    }
    println!();
    println!(
        "{}",
        theme::muted("Waiting for browser connection... (Ctrl+C to stop)")
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
            theme::success("Browser connected! Starting cocoon...")
        );

        // Set environment variables for the cocoon service installation
        std::env::set_var("SIGNALING_SERVER_URL", &req.signaling_url);
        std::env::set_var("COCOON_SETUP_TOKEN", &req.token);
        std::env::set_var("COCOON_NAME", &hostname);

        // Load cocoon plugin to install and start as a service (daemon)
        let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
        runtime.scan_and_load_plugin("adi.cocoon").await?;

        // Install the cocoon service (creates launchd plist or systemd unit)
        let install_context = serde_json::json!({
            "command": "adi.cocoon",
            "args": ["create", "--runtime", "machine", "--start"],
            "cwd": std::env::current_dir().unwrap_or_default().to_string_lossy()
        });

        runtime.run_cli_command("adi.cocoon", &install_context.to_string()).await?;

        println!(
            "{}",
            theme::success("Cocoon installed and running as a background service!")
        );
        println!(
            "  {} {}",
            theme::muted("Status:"),
            theme::brand("adi cocoon status")
        );
        println!(
            "  {} {}",
            theme::muted("Logs:"),
            theme::brand("adi cocoon logs")
        );
        println!(
            "  {} {}",
            theme::muted("Stop:"),
            theme::brand("adi cocoon stop")
        );
    }

    // Abort the HTTP server task
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
    cli::clienv::signaling_url()
}

/// Get a friendly machine name for display
fn get_machine_name() -> String {
    // On macOS, try to get the friendly "Computer Name" first
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("scutil").args(["--get", "ComputerName"]).output() {
            if output.status.success() {
                if let Ok(name) = String::from_utf8(output.stdout) {
                    let name = name.trim();
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
    }
    
    // Fallback to hostname
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Local Machine".to_string())
}

/// Detected capability on the machine
struct Capability {
    name: &'static str,
    category: &'static str,
}

/// Detect available tools/capabilities on the machine
fn detect_capabilities() -> Vec<Capability> {
    use std::process::Command;
    
    let tools: &[(&str, &str)] = &[
        // AI Coding Agents
        ("claude", "ai-agent"),
        ("codex", "ai-agent"),
        ("aider", "ai-agent"),
        ("cursor", "ai-agent"),
        ("copilot", "ai-agent"),
        ("gemini", "ai-agent"),
        // Languages & Runtimes
        ("node", "runtime"),
        ("bun", "runtime"),
        ("deno", "runtime"),
        ("python3", "runtime"),
        ("python", "runtime"),
        ("cargo", "runtime"),
        ("go", "runtime"),
        ("java", "runtime"),
        ("ruby", "runtime"),
        ("php", "runtime"),
        ("dotnet", "runtime"),
        ("swift", "runtime"),
        // Dev Tools
        ("git", "tool"),
        ("gh", "tool"),
        ("docker", "tool"),
        ("kubectl", "tool"),
        ("terraform", "tool"),
        ("aws", "tool"),
        ("gcloud", "tool"),
        ("az", "tool"),
    ];
    
    let mut capabilities = Vec::new();
    
    for (cmd, category) in tools {
        let result = if cfg!(windows) {
            Command::new("where").arg(cmd).output()
        } else {
            Command::new("which").arg(cmd).output()
        };
        
        if let Ok(output) = result {
            if output.status.success() {
                capabilities.push(Capability { name: cmd, category });
            }
        }
    }
    
    capabilities
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
