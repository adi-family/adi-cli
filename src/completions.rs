//! Shell completion generation with dynamic plugin support.
//!
//! It's the glue that makes `adi <Tab>` work in bash/zsh/fish,
//! including plugin commands that aren't compiled into the binary.
//!
//! Generates shell completions that include both static CLI commands
//! and dynamically discovered plugin commands from installed manifests.
//! Used by `adi completions <shell>` and auto-invoked on every CLI run
//! via `ensure_completions_installed` to keep completions up-to-date.

use std::io::Write;
use std::path::PathBuf;

use clap::{Command, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};

/// Supported shells for completion generation.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Elvish,
}

impl From<CompletionShell> for Shell {
    fn from(shell: CompletionShell) -> Self {
        match shell {
            CompletionShell::Bash => Shell::Bash,
            CompletionShell::Zsh => Shell::Zsh,
            CompletionShell::Fish => Shell::Fish,
            CompletionShell::PowerShell => Shell::PowerShell,
            CompletionShell::Elvish => Shell::Elvish,
        }
    }
}

/// Generate shell completions with dynamic plugin commands.
///
/// This builds a clap Command that includes both static commands
/// and plugin-provided commands discovered from manifests.
pub fn generate_completions<C: CommandFactory>(shell: CompletionShell, bin_name: &str) {
    let mut cmd = C::command();

    // Add plugin commands by reading manifests directly (no async needed)
    cmd = add_plugin_commands_from_manifests(cmd);

    // For shells that support dynamic completions, generate enhanced scripts
    match shell {
        CompletionShell::Zsh => {
            generate_zsh_with_dynamic(bin_name, &cmd);
        }
        CompletionShell::Bash => {
            generate_bash_with_dynamic(bin_name, &cmd);
        }
        CompletionShell::Fish => {
            generate_fish_with_dynamic(bin_name, &cmd);
        }
        _ => {
            // Fallback to standard clap completions for other shells
            let shell_type: Shell = shell.into();
            generate(shell_type, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }
}

/// Generate Zsh completions with dynamic plugin support (to stdout)
fn generate_zsh_with_dynamic(bin_name: &str, cmd: &Command) {
    let dynamic_plugins = get_dynamic_completion_plugins();

    if dynamic_plugins.is_empty() {
        // No dynamic plugins, use standard completions
        generate(
            Shell::Zsh,
            &mut cmd.clone(),
            bin_name,
            &mut std::io::stdout(),
        );
        return;
    }

    print!("{}", generate_zsh_script_with_dynamic(bin_name, cmd));
}

/// Generate Bash completions with dynamic plugin support (to stdout)
fn generate_bash_with_dynamic(bin_name: &str, cmd: &Command) {
    let dynamic_plugins = get_dynamic_completion_plugins();

    if dynamic_plugins.is_empty() {
        // No dynamic plugins, use standard completions
        generate(
            Shell::Bash,
            &mut cmd.clone(),
            bin_name,
            &mut std::io::stdout(),
        );
        return;
    }

    print!("{}", generate_bash_script_with_dynamic(bin_name, cmd));
}

/// Generate Fish completions with dynamic plugin support (to stdout)
fn generate_fish_with_dynamic(bin_name: &str, cmd: &Command) {
    let dynamic_plugins = get_dynamic_completion_plugins();

    if dynamic_plugins.is_empty() {
        // No dynamic plugins, use standard completions
        generate(
            Shell::Fish,
            &mut cmd.clone(),
            bin_name,
            &mut std::io::stdout(),
        );
        return;
    }

    print!("{}", generate_fish_script_with_dynamic(bin_name, cmd));
}

/// Track plugins with dynamic completions
static DYNAMIC_COMPLETION_PLUGINS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

/// Get list of plugins that support dynamic completions
pub fn get_dynamic_completion_plugins() -> &'static Vec<String> {
    DYNAMIC_COMPLETION_PLUGINS.get_or_init(Vec::new)
}

/// Discover and add plugin commands by reading manifest files directly.
/// This avoids needing a tokio runtime by reading files synchronously.
fn add_plugin_commands_from_manifests(mut cmd: Command) -> Command {
    use lib_plugin_manifest::PluginManifest;

    let plugins_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("adi")
        .join("plugins");

    if !plugins_dir.exists() {
        return cmd;
    }

    let mut dynamic_plugins = Vec::new();

    // Scan plugins directory for plugin.toml files
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            let plugin_dir = entry.path();
            if !plugin_dir.is_dir() {
                continue;
            }

            // Find plugin.toml manifest
            if let Some(manifest_path) = find_plugin_manifest(&plugin_dir) {
                if let Ok(manifest) = PluginManifest::from_file(&manifest_path) {
                    if let Some(cli) = &manifest.cli {
                        // Leak strings to get 'static lifetime required by clap
                        let name: &'static str = Box::leak(cli.command.clone().into_boxed_str());
                        let desc: &'static str =
                            Box::leak(cli.description.clone().into_boxed_str());

                        let mut subcmd = Command::new(name)
                            .about(desc)
                            .allow_external_subcommands(true);

                        for alias in &cli.aliases {
                            let alias_static: &'static str =
                                Box::leak(alias.clone().into_boxed_str());
                            subcmd = subcmd.visible_alias(alias_static);
                        }

                        // Track if this plugin supports dynamic completions
                        if cli.dynamic_completions {
                            dynamic_plugins.push(cli.command.clone());
                        }

                        cmd = cmd.subcommand(subcmd);
                    }
                }
            }
        }
    }

    // Store dynamic completion plugins for later use
    let _ = DYNAMIC_COMPLETION_PLUGINS.set(dynamic_plugins);

    cmd
}

/// Find the plugin.toml manifest in a plugin directory.
fn find_plugin_manifest(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    // First, check for .version file to get current version
    let version_file = plugin_dir.join(".version");
    if version_file.exists() {
        if let Ok(version) = std::fs::read_to_string(&version_file) {
            let version = version.trim();
            let versioned_manifest = plugin_dir.join(version).join("plugin.toml");
            if versioned_manifest.exists() {
                return Some(versioned_manifest);
            }
        }
    }

    // Fallback: check for plugin.toml directly in plugin dir
    let direct_manifest = plugin_dir.join("plugin.toml");
    if direct_manifest.exists() {
        return Some(direct_manifest);
    }

    // Fallback: scan subdirectories for plugin.toml
    if let Ok(entries) = std::fs::read_dir(plugin_dir) {
        for entry in entries.flatten() {
            let subdir = entry.path();
            if subdir.is_dir() {
                let manifest = subdir.join("plugin.toml");
                if manifest.exists() {
                    return Some(manifest);
                }
            }
        }
    }

    None
}

/// Get the shell configuration file path for the current shell.
pub fn get_shell_config_path(shell: CompletionShell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    match shell {
        CompletionShell::Bash => {
            // Check for .bash_profile first (macOS), then .bashrc (Linux)
            let bash_profile = home.join(".bash_profile");
            let bashrc = home.join(".bashrc");
            if bash_profile.exists() {
                Some(bash_profile)
            } else {
                Some(bashrc)
            }
        }
        CompletionShell::Zsh => Some(home.join(".zshrc")),
        CompletionShell::Fish => Some(home.join(".config/fish/config.fish")),
        CompletionShell::PowerShell => {
            dirs::config_dir().map(|c| c.join("powershell/Microsoft.PowerShell_profile.ps1"))
        }
        CompletionShell::Elvish => Some(home.join(".elvish/rc.elv")),
    }
}

/// Get the completions directory for a shell.
pub fn get_completions_dir(shell: CompletionShell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| home.join(".local/share"));

    match shell {
        CompletionShell::Bash => {
            // Try XDG location first, then fallback
            let xdg = data_dir.join("bash-completion/completions");
            if xdg.parent().map(|p| p.exists()).unwrap_or(false) {
                Some(xdg)
            } else {
                Some(home.join(".bash_completion.d"))
            }
        }
        CompletionShell::Zsh => {
            // Try common zsh completions directories
            let zsh_funcs = home.join(".zfunc");
            Some(zsh_funcs)
        }
        CompletionShell::Fish => Some(home.join(".config/fish/completions")),
        CompletionShell::PowerShell => dirs::config_dir().map(|c| c.join("powershell")),
        CompletionShell::Elvish => Some(home.join(".elvish/lib")),
    }
}

/// Get the completion file name for a shell.
pub fn get_completion_filename(shell: CompletionShell, bin_name: &str) -> String {
    match shell {
        CompletionShell::Bash => format!("{}.bash", bin_name),
        CompletionShell::Zsh => format!("_{}", bin_name),
        CompletionShell::Fish => format!("{}.fish", bin_name),
        CompletionShell::PowerShell => format!("_{}.ps1", bin_name),
        CompletionShell::Elvish => format!("{}.elv", bin_name),
    }
}

/// Initialize shell completions by writing to the appropriate location
/// and updating shell configuration if needed.
pub fn init_completions<C: CommandFactory>(
    shell: CompletionShell,
    bin_name: &str,
) -> anyhow::Result<PathBuf> {
    let completions_dir = get_completions_dir(shell)
        .ok_or_else(|| anyhow::anyhow!("Could not determine completions directory"))?;

    // Create directory if it doesn't exist
    std::fs::create_dir_all(&completions_dir)?;

    let completion_file = completions_dir.join(get_completion_filename(shell, bin_name));

    // Generate completions to file
    let file = std::fs::File::create(&completion_file)?;
    let mut cmd = C::command();

    // Add plugin commands (sync version, no runtime needed)
    cmd = add_plugin_commands_from_manifests(cmd);

    // Generate with dynamic completion support
    write_completions_to_file(shell, bin_name, &cmd, file)?;

    // For some shells, we need to update the rc file
    match shell {
        CompletionShell::Zsh => {
            add_to_shell_config(
                shell,
                r#"
# ADI CLI completions
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
"#,
            )?;
        }
        CompletionShell::Bash => {
            let source_line = format!("source \"{}\"", completion_file.display());
            add_to_shell_config(
                shell,
                &format!(
                    r#"
# ADI CLI completions
{}
"#,
                    source_line
                ),
            )?;
        }
        CompletionShell::Fish => {
            // Fish auto-loads from ~/.config/fish/completions
        }
        _ => {}
    }

    Ok(completion_file)
}

/// Write completions to a file with dynamic plugin support
fn write_completions_to_file(
    shell: CompletionShell,
    bin_name: &str,
    cmd: &Command,
    mut file: std::fs::File,
) -> anyhow::Result<()> {
    use std::io::Write;

    let dynamic_plugins = get_dynamic_completion_plugins();

    match shell {
        CompletionShell::Zsh if !dynamic_plugins.is_empty() => {
            let script = generate_zsh_script_with_dynamic(bin_name, cmd);
            file.write_all(script.as_bytes())?;
        }
        CompletionShell::Bash if !dynamic_plugins.is_empty() => {
            let script = generate_bash_script_with_dynamic(bin_name, cmd);
            file.write_all(script.as_bytes())?;
        }
        CompletionShell::Fish if !dynamic_plugins.is_empty() => {
            let script = generate_fish_script_with_dynamic(bin_name, cmd);
            file.write_all(script.as_bytes())?;
        }
        _ => {
            let shell_type: Shell = shell.into();
            generate(shell_type, &mut cmd.clone(), bin_name, &mut file);
        }
    }

    Ok(())
}

/// Generate Zsh script as a String (for file writing)
fn generate_zsh_script_with_dynamic(bin_name: &str, cmd: &Command) -> String {
    let dynamic_plugins = get_dynamic_completion_plugins();
    let mut script = String::new();

    script.push_str(&format!(
        r#"#compdef {bin_name}

# Dynamic completion function for plugins
_adi_dynamic_complete() {{
    local cmd=$1
    local pos=$2
    shift 2
    local words=("$@")
    
    # Call the plugin's --completions command
    local completions
    completions=$({bin_name} "$cmd" --completions "$pos" "${{words[@]}}" 2>/dev/null)
    
    if [[ -n "$completions" ]]; then
        local -a comp_array
        while IFS=$'\t' read -r comp desc; do
            if [[ -n "$desc" ]]; then
                comp_array+=("$comp:$desc")
            else
                comp_array+=("$comp")
            fi
        done <<< "$completions"
        _describe -t completions 'completions' comp_array
        return 0
    fi
    return 1
}}

_adi() {{
    local context state state_descr line
    typeset -A opt_args

    _arguments -C \
        '1: :->command' \
        '*::arg:->args'

    case $state in
        command)
            local -a commands
            commands=(
                'plugin:Manage plugins'
                'search:Search packages'
                'services:List services'
                'run:Run a plugin command'
                'self-update:Update adi CLI'
                'completions:Generate shell completions'
"#
    ));

    // Add plugin commands
    for subcmd in cmd.get_subcommands() {
        let name = subcmd.get_name();
        let about = subcmd
            .get_about()
            .map(|s| s.to_string())
            .unwrap_or_default();
        if ![
            "plugin",
            "search",
            "services",
            "run",
            "self-update",
            "completions",
        ]
        .contains(&name)
        {
            script.push_str(&format!("                '{name}:{about}'\n"));
        }
    }

    script.push_str(
        r#"            )
            _describe -t commands 'adi commands' commands
            ;;
        args)
            case $line[1] in
"#,
    );

    for plugin_cmd in dynamic_plugins {
        script.push_str(&format!(
            r#"                {plugin_cmd})
                    _adi_dynamic_complete "{plugin_cmd}" $((CURRENT)) "${{words[@]:1}}"
                    ;;
"#
        ));
    }

    script.push_str(
        r#"                *)
                    _files
                    ;;
            esac
            ;;
    esac
}

_adi "$@"
"#,
    );

    script
}

/// Generate Bash script as a String (for file writing)
fn generate_bash_script_with_dynamic(bin_name: &str, cmd: &Command) -> String {
    let dynamic_plugins = get_dynamic_completion_plugins();
    let subcommands: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();
    let subcommands_str = subcommands.join(" ");
    let dynamic_str = dynamic_plugins.join("|");

    format!(
        r#"# Bash completion for {bin_name}

_{bin_name}_dynamic_complete() {{
    local cmd=$1
    local pos=$2
    shift 2
    local words=("$@")
    
    # Call the plugin's --completions command
    local completions
    completions=$({bin_name} "$cmd" --completions "$pos" "${{words[@]}}" 2>/dev/null)
    
    if [[ -n "$completions" ]]; then
        # Parse tab-separated completions (completion\tdescription)
        local -a comps
        while IFS=$'\t' read -r comp desc; do
            comps+=("$comp")
        done <<< "$completions"
        COMPREPLY=($(compgen -W "${{comps[*]}}" -- "${{COMP_WORDS[COMP_CWORD]}}"))
        return 0
    fi
    return 1
}}

_{bin_name}() {{
    local cur prev words cword
    _init_completion || return

    local commands="{subcommands_str}"

    if [[ $cword -eq 1 ]]; then
        COMPREPLY=($(compgen -W "$commands" -- "$cur"))
        return
    fi

    local cmd="${{words[1]}}"
    
    case "$cmd" in
        {dynamic_str})
            # Dynamic completion for these commands
            local pos=$((cword - 1))
            local cmd_words=("${{words[@]:2}}")
            _{bin_name}_dynamic_complete "$cmd" "$pos" "${{cmd_words[@]}}"
            ;;
        *)
            # Default file completion
            _filedir
            ;;
    esac
}}

complete -F _{bin_name} {bin_name}
"#
    )
}

/// Generate Fish script as a String (for file writing)
fn generate_fish_script_with_dynamic(bin_name: &str, cmd: &Command) -> String {
    let dynamic_plugins = get_dynamic_completion_plugins();
    let mut script = String::new();

    script.push_str(&format!(
        r#"# Fish completion for {bin_name}

# Dynamic completion function
function __adi_dynamic_complete
    set -l cmd $argv[1]
    set -l pos $argv[2]
    set -l words $argv[3..-1]
    
    # Call the plugin's --completions command  
    set -l completions ({bin_name} $cmd --completions $pos $words 2>/dev/null)
    
    for line in $completions
        # Parse tab-separated: completion\tdescription
        set -l parts (string split \t $line)
        if test (count $parts) -ge 2
            echo $parts[1]\t$parts[2]
        else
            echo $parts[1]
        end
    end
end

# Disable file completions for adi
complete -c {bin_name} -f
"#
    ));

    // Add static subcommand completions
    for subcmd in cmd.get_subcommands() {
        let name = subcmd.get_name();
        let about = subcmd
            .get_about()
            .map(|s| s.to_string())
            .unwrap_or_default();
        script.push_str(&format!(
            r#"complete -c {bin_name} -n "__fish_use_subcommand" -a "{name}" -d "{about}"
"#
        ));

        for alias in subcmd.get_visible_aliases() {
            script.push_str(&format!(
                r#"complete -c {bin_name} -n "__fish_use_subcommand" -a "{alias}" -d "{about}"
"#
            ));
        }
    }

    script.push('\n');

    // Add dynamic completions for supported plugins
    for plugin_cmd in dynamic_plugins {
        script.push_str(&format!(
            r#"# Dynamic completions for {plugin_cmd}
complete -c {bin_name} -n "__fish_seen_subcommand_from {plugin_cmd}" -a "(__adi_dynamic_complete {plugin_cmd} (count (commandline -opc)) (commandline -opc)[3..-1])"
"#
        ));
    }

    script
}

/// Add a configuration snippet to the shell config file if not already present.
fn add_to_shell_config(shell: CompletionShell, snippet: &str) -> anyhow::Result<()> {
    let config_path = get_shell_config_path(shell)
        .ok_or_else(|| anyhow::anyhow!("Could not determine shell config path"))?;

    // Read existing config
    let existing = std::fs::read_to_string(&config_path).unwrap_or_default();

    // Check if ADI completions are already configured
    if existing.contains("# ADI CLI completions") {
        return Ok(());
    }

    // Append to config
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)?;

    writeln!(file, "{}", snippet)?;

    Ok(())
}

/// Regenerate completions (called after plugin install/uninstall).
pub fn regenerate_completions<C: CommandFactory>(bin_name: &str) -> anyhow::Result<()> {
    // Try to regenerate for all shells that have completions installed
    for shell in [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
    ] {
        if let Some(dir) = get_completions_dir(shell) {
            let file_path = dir.join(get_completion_filename(shell, bin_name));
            if file_path.exists() {
                // Regenerate this completion file
                let file = std::fs::File::create(&file_path)?;
                let mut cmd = C::command();

                // Add plugin commands (sync version, no runtime needed)
                cmd = add_plugin_commands_from_manifests(cmd);

                // Use the new dynamic-aware writing function
                write_completions_to_file(shell, bin_name, &cmd, file)?;
            }
        }
    }

    Ok(())
}

/// Detect the current shell from environment.
pub fn detect_shell() -> Option<CompletionShell> {
    std::env::var("SHELL").ok().and_then(|s| {
        if s.contains("zsh") {
            Some(CompletionShell::Zsh)
        } else if s.contains("bash") {
            Some(CompletionShell::Bash)
        } else if s.contains("fish") {
            Some(CompletionShell::Fish)
        } else if s.contains("pwsh") || s.contains("powershell") {
            Some(CompletionShell::PowerShell)
        } else if s.contains("elvish") {
            Some(CompletionShell::Elvish)
        } else {
            None
        }
    })
}

/// Ensure shell completions are installed (called automatically on every run).
/// This is idempotent and optimized - only regenerates when plugins change.
pub fn ensure_completions_installed<C: CommandFactory>(bin_name: &str) {
    let Some(shell) = detect_shell() else {
        return;
    };

    let Some(completions_dir) = get_completions_dir(shell) else {
        return;
    };

    let completion_file = completions_dir.join(get_completion_filename(shell, bin_name));
    let marker_file = completions_dir.join(format!(".{}-installed", bin_name));

    // Check if we need to regenerate completions
    let needs_shell_config = !marker_file.exists();
    let needs_regenerate = needs_shell_config || completions_outdated(&completion_file);

    if !needs_regenerate {
        return;
    }

    // Create completions directory
    if std::fs::create_dir_all(&completions_dir).is_err() {
        return;
    }

    // Generate completions
    let Ok(file) = std::fs::File::create(&completion_file) else {
        return;
    };

    let mut cmd = C::command();
    cmd = add_plugin_commands_from_manifests(cmd);

    // Use dynamic-aware completion writing
    let _ = write_completions_to_file(shell, bin_name, &cmd, file);

    // First time setup: update shell config
    if needs_shell_config {
        let _ = setup_shell_config(shell, &completion_file);
        // Create marker file
        let _ = std::fs::write(&marker_file, "");
    }
}

/// Check if completions file is older than the plugins directory.
fn completions_outdated(completion_file: &std::path::Path) -> bool {
    let plugins_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("adi")
        .join("plugins");

    // If plugins dir doesn't exist, no need to regenerate
    if !plugins_dir.exists() {
        return false;
    }

    // If completion file doesn't exist, need to generate
    let Ok(completion_meta) = std::fs::metadata(completion_file) else {
        return true;
    };

    let Ok(completion_time) = completion_meta.modified() else {
        return true;
    };

    // Check if any plugin dir is newer than completion file
    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified > completion_time {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Set up shell configuration to source completions.
fn setup_shell_config(
    shell: CompletionShell,
    completion_file: &std::path::Path,
) -> anyhow::Result<()> {
    match shell {
        CompletionShell::Zsh => {
            add_to_shell_config(
                shell,
                r#"
# ADI CLI completions
fpath=(~/.zfunc $fpath)
autoload -Uz compinit && compinit
"#,
            )?;
        }
        CompletionShell::Bash => {
            let source_line = format!("source \"{}\"", completion_file.display());
            add_to_shell_config(
                shell,
                &format!(
                    r#"
# ADI CLI completions
{}
"#,
                    source_line
                ),
            )?;
        }
        CompletionShell::Fish => {
            // Fish auto-loads from ~/.config/fish/completions
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell() {
        // This test depends on the environment
        let shell = detect_shell();
        // Just verify it doesn't panic
        println!("Detected shell: {:?}", shell);
    }

    #[test]
    fn test_completion_filename() {
        assert_eq!(
            get_completion_filename(CompletionShell::Bash, "adi"),
            "adi.bash"
        );
        assert_eq!(get_completion_filename(CompletionShell::Zsh, "adi"), "_adi");
        assert_eq!(
            get_completion_filename(CompletionShell::Fish, "adi"),
            "adi.fish"
        );
    }
}
