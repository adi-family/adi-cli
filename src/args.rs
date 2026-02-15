use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "adi")]
#[command(version)]
#[command(about = "CLI for ADI family components", long_about = None)]
pub(crate) struct Cli {
    /// Override language (e.g., en-US, zh-CN). Can also be set via ADI_LANG env var.
    #[arg(long, global = true)]
    pub lang: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
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

    /// Run a plugin's CLI interface
    Run {
        /// Plugin ID to run (shows available plugins if omitted)
        plugin_id: Option<String>,

        /// Arguments to pass to the plugin
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
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
pub(crate) enum PluginCommands {
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
