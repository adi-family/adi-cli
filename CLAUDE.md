adi-cli, rust, component-manager, github-releases, cross-platform

## Overview
- ADI CLI Manager - installs and manages ADI family components
- Binary name: `adi` (run as `adi <command>`)
- License: BSL-1.0

## Commands
- `adi list` - List available components
- `adi install <component>` - Install a component
- `adi uninstall <component>` - Uninstall a component
- `adi update [component]` - Update component(s)
- `adi status` - Show installed component status

## Architecture
- Plugin-based component system via `Component` trait
- Components defined in `src/components/`
- Registry in `src/components/mod.rs` via `create_default_registry()`
- `Component` trait requires: `info()`, `status()`, `install()`, `uninstall()`
- Components install to `~/.local/share/adi/` with binaries in `~/.local/share/adi/bin/`

## Deployment
- Tag-based GitHub Actions: `cli-v{VERSION}`
- Cross-platform: macOS (Intel/ARM), Linux (x86_64), Windows (x86_64)
