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
