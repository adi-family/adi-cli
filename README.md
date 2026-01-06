# ADI CLI - Knowledgebase

Facts and code references for landing page generation.

## Identity

- **Binary name**: `adi` ([Cargo.toml:10](src/main.rs))
- **Full name**: ADI CLI Manager
- **Version**: 0.8.8 ([Cargo.toml:3](Cargo.toml))
- **License**: BSL-1.0 ([Cargo.toml:5](Cargo.toml))
- **Repository**: https://github.com/adi-family/adi-cli ([config.toml:9](config.toml))
- **Registry URL**: https://adi-plugin-registry.the-ihor.com ([config.toml:13](config.toml))

## Core Purpose

- Plugin manager and installer for the ADI ecosystem
- Downloads, installs, updates, and uninstalls plugins from remote registry
- Provides unified CLI entry point for all ADI family tools

## Supported Platforms

- macOS (Intel x86_64 and ARM aarch64)
- Linux (x86_64)
- Windows (x86_64)

Platform detection logic: [plugin_registry.rs:59-78](src/plugin_registry.rs)

## Commands

| Command | Description | Code Reference |
|---------|-------------|----------------|
| `adi self-update` | Update CLI to latest version | [main.rs:29-33](src/main.rs) |
| `adi plugin list` | List all available plugins | [main.rs:88](src/main.rs) |
| `adi plugin installed` | List installed plugins | [main.rs:91](src/main.rs) |
| `adi plugin install <id>` | Install a plugin | [main.rs:94-101](src/main.rs) |
| `adi plugin update <id>` | Update a plugin | [main.rs:103-107](src/main.rs) |
| `adi plugin update-all` | Update all plugins | [main.rs:109-110](src/main.rs) |
| `adi plugin uninstall <id>` | Uninstall a plugin | [main.rs:113-116](src/main.rs) |
| `adi search <query>` | Search registry | [main.rs:42-45](src/main.rs) |
| `adi services` | List registered services | [main.rs:48](src/main.rs) |
| `adi run <plugin>` | Run a plugin's CLI | [main.rs:50-58](src/main.rs) |
| `adi completions <shell>` | Generate shell completions | [main.rs:60-65](src/main.rs) |
| `adi init <shell>` | Initialize shell completions | [main.rs:67-72](src/main.rs) |

## Language Support (i18n)

8 languages supported: [main.rs:220-229](src/main.rs)

| Language | Code |
|----------|------|
| English | `en-US` |
| Chinese (Simplified) | `zh-CN` |
| Ukrainian | `uk-UA` |
| Spanish | `es-ES` |
| French | `fr-FR` |
| German | `de-DE` |
| Japanese | `ja-JP` |
| Korean | `ko-KR` |

### Language Detection Priority

1. `--lang` CLI flag
2. `ADI_LANG` environment variable
3. Saved user preference (`~/.config/adi/config.toml`)
4. System `LANG` environment variable
5. Interactive prompt on first run (TTY only)
6. Default: `en-US`

Detection logic: [main.rs:252-290](src/main.rs)

### Auto-Install

Missing language plugins are automatically installed: [main.rs:297-318](src/main.rs)

## Shell Completion Support

5 shells supported: [completions.rs](src/completions.rs)

- Bash
- Zsh
- Fish
- PowerShell
- Elvish

### Features

- Auto-detection of current shell
- Auto-installation on first run: [main.rs:122](src/main.rs)
- Auto-regeneration when plugins change: [main.rs:440-452](src/main.rs)
- Dynamic plugin commands included in completions

## Plugin System

### Architecture

```
Plugin Registry (remote)
    ↓ download
~/.local/share/adi/plugins/
    ↓ load
lib-plugin-host (binary loading)
    ↓ expose
Service Registry (JSON-RPC)
```

### Plugin Storage

- **Plugins directory**: `~/.local/share/adi/plugins/`
- **Cache directory**: `~/.local/share/adi/cache/`
- **Config file**: `~/.config/adi/config.toml`

Directory configuration: [plugin_runtime.rs:31-49](src/plugin_runtime.rs)

### Plugin Structure

```
plugins/
├── <plugin-id>/
│   ├── .version          # Current installed version
│   └── <version>/
│       ├── plugin.toml   # Plugin manifest
│       └── <binary>      # Executable
```

Manifest discovery: [plugin_runtime.rs:264-343](src/plugin_runtime.rs)

### Dependency Resolution

- Recursive dependency installation
- Cycle detection with HashSet
- Already-installed plugins skipped

Implementation: [plugin_registry.rs:177-238](src/plugin_registry.rs)

### Pattern Matching Installation

Supports glob patterns for bulk installation:
- `adi plugin install "adi.lang.*"` - Install all language plugins
- `adi plugin install "adi.*"` - Install all ADI plugins

Implementation: [plugin_registry.rs:367-459](src/plugin_registry.rs)

## Self-Update

### Release Detection

Fetches from GitHub releases with tag format: `cli-v{version}` or `v{version}` (legacy)

Logic: [self_update.rs:98-117](src/self_update.rs)

### Platform Binary Names

- macOS: `adi-apple-darwin-{arch}.tar.gz`
- Linux: `adi-unknown-linux-gnu-{arch}.tar.gz`
- Windows: `adi-pc-windows-msvc-{arch}.zip`

### Version Comparison

Semantic versioning comparison: [self_update.rs:240-258](src/self_update.rs)

## Available Components

From [config.toml:35-97](config.toml):

| Component | Binary | Description |
|-----------|--------|-------------|
| indexer-cli | `adi-indexer-cli` | CLI for ADI Code Indexer |
| indexer-http | `adi-indexer-http` | HTTP API for Code Indexer |
| indexer-mcp | `adi-indexer-mcp` | MCP server for Code Indexer |
| tasks-cli | `adi-tasks-cli` | CLI for ADI Tasks |
| tasks-http | `adi-tasks-http` | HTTP API for ADI Tasks |
| tasks-mcp | `adi-tasks-mcp` | MCP server for ADI Tasks |
| tarminal | `tarminal` | GPU-accelerated terminal (macOS) |

## Key Dependencies

From [Cargo.toml:13-42](Cargo.toml):

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `clap` | CLI framework |
| `reqwest` | HTTP client |
| `serde` / `toml` | Serialization |
| `indicatif` | Progress bars |
| `dialoguer` | Interactive prompts |
| `console` | Terminal styling |
| `lib-plugin-host` | Plugin loading |
| `lib-plugin-registry` | Registry client |
| `lib-i18n-core` | Internationalization |

## Internal Crates

- `lib-plugin-registry` - Plugin discovery and download
- `lib-plugin-host` - Binary loading and service registry
- `lib-plugin-abi` - Service interface definitions
- `lib-plugin-manifest` - TOML manifest parsing
- `lib-i18n-core` - Translation system with `t!()` macro
- `lib-client-github` - GitHub API for release fetching

## Service Registry

Plugins expose services via JSON-RPC interface:

- **CLI services**: `{plugin-id}.cli` - Command execution
- **HTTP routes**: HTTP request handling
- **Translation services**: `translation.{namespace}` - i18n

Service listing: [main.rs:511-542](src/main.rs)

## Error Handling

Error types defined in [error.rs](src/error.rs):

- `ComponentNotFound` - Requested component doesn't exist
- `InstallationFailed` - Plugin installation failed
- `DependencyMissing` - Required dependency not available
- `AlreadyInstalled` - Plugin already at latest version
- `PluginNotFound` - Plugin not in registry
- `PluginHost` - Dynamic library loading error

## Interactive First-Run

On first interactive run:
1. Welcome message displayed
2. Language selection prompt shown
3. User preference saved to config
4. Language plugin auto-installed

Implementation: [main.rs:231-290](src/main.rs)

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ADI_REGISTRY_URL` | Override plugin registry URL |
| `ADI_LANG` | Set language (e.g., `en-US`) |

## Source File Summary

| File | Lines | Purpose |
|------|-------|---------|
| `main.rs` | 710 | CLI entry point, commands |
| `plugin_runtime.rs` | 383 | Plugin lifecycle management |
| `plugin_registry.rs` | 518 | Registry client, installation |
| `completions.rs` | 444 | Shell completion generation |
| `self_update.rs` | 275 | CLI self-update mechanism |
| `user_config.rs` | 70 | User preferences |
| `project_config.rs` | 130 | Embedded component registry |
| `error.rs` | 55 | Error types |
| `lib.rs` | 13 | Module exports |
