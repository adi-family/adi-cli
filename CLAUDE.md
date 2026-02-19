adi-cli, rust, plugin-manager, plugin-registry, cross-platform

## Overview
- ADI CLI Manager - installs and manages plugins from registry
- Binary name: `adi` (run as `adi <command>`)
- Plugin registry: https://adi-plugin-registry.the-ihor.com
- License: BSL-1.0

## Commands
- `adi search <query>` - Search plugins/packages in registry
- `adi plugin list` - List all available plugins from registry
- `adi plugin installed` - List installed plugins
- `adi plugin install <plugin-id>` - Install a plugin
- `adi plugin update <plugin-id>` - Update a plugin
- `adi plugin update-all` - Update all installed plugins
- `adi plugin uninstall <plugin-id>` - Uninstall a plugin
- `adi services` - List registered services from loaded plugins
- `adi run [plugin-id]` - Run a plugin's CLI interface (lists runnable plugins if omitted)
- `adi self-update` - Update adi CLI itself
- `adi config` - Interactive config editor (TTY) or show config (non-TTY)
- `adi config show` - Show current configuration
- `adi config power-user <true|false>` - Enable or disable power user mode

## Architecture
- Plugin-based system using dynamic libraries (cdylib)
- Plugin loading via `lib-plugin-host` crate
- Service registry for inter-plugin communication (JSON-RPC)
- CLI delegates to `adi.cli.commands` services
- Plugins install to `~/.local/share/adi/plugins/`

## Key Files
- `src/plugin_runtime.rs` - PluginRuntime wrapping PluginHost
- `src/plugin_registry.rs` - Plugin download/management

## Internationalization (i18n)
- First launch in interactive session prompts for preferred language
- Non-interactive sessions use defaults until preference set
- Language preference stored in `~/.config/adi/config.toml`
- Available languages: English, 中文, Українська, Español, Français, Deutsch, 日本語, 한국語
- **Auto-install**: Missing language plugins are automatically installed from registry

### Language Selection Priority
1. `--lang` CLI flag (highest priority)
2. `ADI_LANG` environment variable
3. Saved user preference in config file
4. System `LANG` environment variable
5. Interactive prompt on first run (if TTY)
6. Default to `en-US`

### User Config
- Location: `~/.config/adi/config.toml`
- Format: TOML with user preferences (language, theme, power_user)
- Auto-created on first interactive run when language is selected

### Power User Mode
- Enables advanced features and verbose output
- Set via: `adi config power-user true` or `ADI_POWER_USER=true`
- Check with: `cli::clienv::is_power_user()` (env var > config > default false)

## Environment Variables
- `ADI_REGISTRY_URL` - Override default plugin registry URL
- `ADI_LANG` - Set language (e.g., `en-US`, `zh-CN`, `uk-UA`)
- `ADI_POWER_USER` - Enable power user mode (true/false)

## Deployment
- Cross-platform: macOS (Intel/ARM), Linux (x86_64), Windows (x86_64)
