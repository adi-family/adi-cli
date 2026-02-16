# ============================================================================
# SELF-UPDATE DOMAIN
# ============================================================================

self-update-checking = Checking for updates...
self-update-already-latest = You are already on the latest version ({ $version })
self-update-new-version = New version available: { $current } → { $latest }
self-update-downloading = Downloading update...
self-update-extracting = Extracting update...
self-update-installing = Installing update...
self-update-success = Successfully updated to version { $version }
self-update-error-platform = Unsupported operating system
self-update-error-arch = Unsupported architecture
self-update-error-no-asset = No release asset found for platform: { $platform }
self-update-error-no-release = No CLI manager release found

# ============================================================================
# SHELL COMPLETIONS DOMAIN
# ============================================================================

completions-init-start = Initializing shell completions for { $shell }...
completions-init-done = Done! Completions installed to: { $path }
completions-restart-zsh = Restart your shell or run: source ~/.zshrc
completions-restart-bash = Restart your shell or run: source ~/.bashrc
completions-restart-fish = Completions are active immediately in new fish sessions.
completions-restart-generic = Restart your shell to enable completions.
completions-error-no-shell = Could not detect shell. Please specify: adi init bash|zsh|fish

# ============================================================================
# PLUGIN MANAGEMENT DOMAIN
# ============================================================================

# Plugin listing
plugin-list-title = Available Plugins:
plugin-list-empty = No plugins available in the registry.
plugin-installed-title = Installed Plugins:
plugin-installed-empty = No plugins installed.
plugin-installed-hint = Install plugins with: adi plugin install <plugin-id>

# Plugin installation
plugin-install-downloading = Downloading { $id } v{ $version } for { $platform }...
plugin-install-extracting = Extracting to { $path }...
plugin-install-success = Installed { $id } v{ $version } successfully!
plugin-install-already-installed = { $id } v{ $version } is already installed
plugin-install-dependency = Installing dependency: { $id }
plugin-install-error-platform = Plugin { $id } does not support platform { $platform }
plugin-install-pattern-searching = Searching for plugins matching pattern "{ $pattern }"...
plugin-install-pattern-found = Found { $count } plugin(s) matching pattern
plugin-install-pattern-none = No plugins found matching pattern "{ $pattern }"
plugin-install-pattern-installing = Installing { $count } plugin(s)...
plugin-install-pattern-success = { $count } plugin(s) installed successfully!
plugin-install-pattern-failed = Failed to install:

# Plugin updates
plugin-update-checking = Checking for updates to { $id }...
plugin-update-already-latest = { $id } is already at latest version ({ $version })
plugin-update-available = Updating { $id } from { $current } to { $latest }...
plugin-update-downloading = Downloading { $id } v{ $version }...
plugin-update-success = Updated { $id } to v{ $version }
plugin-update-all-start = Updating { $count } plugin(s)...
plugin-update-all-done = Update complete!
plugin-update-all-warning = Failed to update { $id }: { $error }

# Plugin uninstallation
plugin-uninstall-prompt = Uninstall plugin { $id }?
plugin-uninstall-cancelled = Cancelled.
plugin-uninstall-progress = Uninstalling { $id }...
plugin-uninstall-success = { $id } uninstalled successfully!
plugin-uninstall-error-not-installed = Plugin { $id } is not installed

# ============================================================================
# SEARCH DOMAIN
# ============================================================================

search-searching = Searching for "{ $query }"...
search-no-results = No results found.
search-packages-title = Packages:
search-plugins-title = Plugins:
search-results-summary = Found { $packages } package(s) and { $plugins } plugin(s)

# ============================================================================
# SERVICES DOMAIN
# ============================================================================

services-title = Registered Services:
services-empty = No services registered.
services-hint = Install plugins to add services: adi plugin install <id>

# ============================================================================
# RUN COMMAND DOMAIN
# ============================================================================

run-title = Runnable Plugins:
run-empty = No plugins with CLI interface installed.
run-hint-install = Install plugins with: adi plugin install <plugin-id>
run-hint-usage = Run a plugin with: adi run <plugin-id> [args...]
run-error-not-found = Plugin '{ $id }' not found or has no CLI interface
run-error-no-plugins = No runnable plugins installed.
run-error-available = Runnable plugins:
run-error-failed = Failed to run plugin: { $error }

# ============================================================================
# EXTERNAL COMMANDS DOMAIN
# ============================================================================

external-error-no-command = No command provided
external-error-unknown = Unknown command: { $command }
external-error-no-installed = No plugin commands installed.
external-hint-install = Install plugins with: adi plugin install <plugin-id>
external-available-title = Available plugin commands:
external-error-load-failed = Failed to load plugin '{ $id }': { $error }
external-hint-reinstall = Try reinstalling: adi plugin install { $id }
external-error-run-failed = Failed to run { $command }: { $error }

# Auto-install
external-autoinstall-found = Plugin '{ $id }' provides command '{ $command }'
external-autoinstall-prompt = Would you like to install it? [y/N]
external-autoinstall-installing = Installing plugin '{ $id }'...
external-autoinstall-success = Plugin installed successfully!
external-autoinstall-failed = Failed to install plugin: { $error }
external-autoinstall-disabled = Auto-install disabled. Run: adi plugin install { $id }
external-autoinstall-not-found = No plugin found providing command '{ $command }'

# ============================================================================
# INFO COMMAND
# ============================================================================

info-title = ADI CLI Info
info-version = Version
info-config-dir = Config
info-plugins-dir = Plugins
info-registry = Registry
info-theme = Theme
info-language = Language
info-installed-plugins = Installed Plugins ({ $count })
info-no-plugins = No plugins installed
info-commands-title = Commands
info-plugin-commands = Plugin commands:
info-cmd-info = Show CLI info, version, and paths
info-cmd-start = Start local ADI server
info-cmd-plugin = Manage plugins
info-cmd-run = Run a plugin CLI
info-cmd-logs = Stream plugin logs
info-cmd-self-update = Update adi CLI

# ============================================================================
# INTERACTIVE COMMAND SELECTION
# ============================================================================

interactive-select-command = Choose a command

# Command labels
interactive-cmd-info = info
interactive-cmd-start = start
interactive-cmd-plugin = plugin
interactive-cmd-search = search
interactive-cmd-run = run
interactive-cmd-logs = logs
interactive-cmd-debug = debug
interactive-cmd-self-update = self-update
interactive-cmd-completions = completions
interactive-cmd-init = init

# Command descriptions
interactive-cmd-info-desc = Show CLI info, version, paths, and installed plugins
interactive-cmd-start-desc = Start local ADI server for browser connection
interactive-cmd-plugin-desc = Manage plugins from the registry
interactive-cmd-search-desc = Search for plugins and packages
interactive-cmd-run-desc = Run a plugin's CLI interface
interactive-cmd-logs-desc = Stream live logs from a plugin
interactive-cmd-debug-desc = Debug and diagnostic commands
interactive-cmd-self-update-desc = Update adi CLI to the latest version
interactive-cmd-completions-desc = Generate shell completions
interactive-cmd-init-desc = Initialize shell completions

# Argument prompts
interactive-self-update-force = Force update even if on latest version?
interactive-start-port = Port
interactive-search-query = Search query
interactive-completions-shell = Select shell
interactive-init-shell = Select shell (leave empty to auto-detect)
interactive-logs-plugin-id = Plugin ID (e.g., adi.hive)
interactive-logs-follow = Follow log output?
interactive-logs-lines = Number of lines

# Plugin subcommand prompts
interactive-plugin-select = Select plugin action
interactive-plugin-list = List available
interactive-plugin-installed = List installed
interactive-plugin-search = Search
interactive-plugin-install = Install
interactive-plugin-update = Update
interactive-plugin-update-all = Update all
interactive-plugin-uninstall = Uninstall
interactive-plugin-path = Show path
interactive-plugin-install-id = Plugin ID to install (e.g., adi.tasks)
interactive-plugin-update-id = Plugin ID to update
interactive-plugin-uninstall-id = Plugin ID to uninstall
interactive-plugin-path-id = Plugin ID

# ============================================================================
# COMMON/SHARED MESSAGES
# ============================================================================

common-version-prefix = v
common-tags-label = Tags:
common-error-prefix = Error:
common-warning-prefix = Warning:
common-info-prefix = Info:
common-success-prefix = Success:
common-downloading-prefix = →
common-checkmark = ✓
common-arrow = →

# ============================================================================
# ERRORS DOMAIN
# ============================================================================

error-component-not-found = Component '{ $name }' not found
error-installation-failed = Installation failed for '{ $component }': { $reason }
error-dependency-missing = Dependency '{ $dependency }' required by '{ $component }' is not installed
error-config = Configuration error: { $detail }
error-io = IO error: { $detail }
error-serialization = Serialization error: { $detail }
error-already-installed = Component '{ $name }' is already installed
error-uninstallation-failed = Uninstallation failed for '{ $component }': { $reason }
error-registry = Registry error: { $detail }
error-plugin-not-found = Plugin not found: { $id }
error-plugin-host = Plugin host error: { $detail }
error-service = Service error: { $detail }
error-other = Error: { $detail }
