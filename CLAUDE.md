adi-cli, rust, plugin-manager, plugin-registry, mcp-server, http-server, cross-platform

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
- `adi mcp` - Start MCP server (JSON-RPC over stdio)
- `adi http` - Start HTTP server for plugin-provided routes
- `adi services` - List registered services from loaded plugins
- `adi run <plugin-id>` - Run a plugin's CLI interface
- `adi self-update` - Update adi CLI itself

## Architecture
- Plugin-based system using dynamic libraries (cdylib)
- Plugin loading via `lib-plugin-host` crate
- Service registry for inter-plugin communication (JSON-RPC)
- MCP server dispatches to `adi.mcp.tools` and `adi.mcp.resources` services
- HTTP server dispatches to `adi.http.routes` services
- CLI delegates to `adi.cli.commands` services
- Plugins install to `~/.local/share/adi/plugins/`

## Key Files
- `src/plugin_runtime.rs` - PluginRuntime wrapping PluginHost
- `src/mcp_server.rs` - MCP JSON-RPC server over stdio
- `src/http_server.rs` - Axum HTTP server
- `src/plugin_registry.rs` - Plugin download/management

## Environment Variables
- `ADI_REGISTRY_URL` - Override default plugin registry URL

## Deployment
- Tag-based GitHub Actions: `cli-v{VERSION}`
- Cross-platform: macOS (Intel/ARM), Linux (x86_64), Windows (x86_64)
