use std::io::Write;
use std::path::PathBuf;

use clap::{Command, CommandFactory, ValueEnum};
use clap_complete::{generate, Shell};

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

pub fn generate_completions<C: CommandFactory>(shell: CompletionShell, bin_name: &str) {
    tracing::trace!(shell = ?shell, bin_name = %bin_name, "Generating shell completions");
    let mut cmd = C::command();
    cmd = add_plugin_commands_from_manifests(cmd);

    let dynamic_plugins = get_dynamic_completion_plugins();
    let has_dynamic = !dynamic_plugins.is_empty();

    let script = match (shell, has_dynamic) {
        (CompletionShell::Zsh, true) => Some(generate_zsh_script_with_dynamic(bin_name, &cmd)),
        (CompletionShell::Bash, true) => Some(generate_bash_script_with_dynamic(bin_name, &cmd)),
        (CompletionShell::Fish, true) => Some(generate_fish_script_with_dynamic(bin_name, &cmd)),
        _ => None,
    };

    match script {
        Some(s) => print!("{}", s),
        None => {
            let shell_type: Shell = shell.into();
            generate(shell_type, &mut cmd, bin_name, &mut std::io::stdout());
        }
    }
}

static DYNAMIC_COMPLETION_PLUGINS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

pub fn get_dynamic_completion_plugins() -> &'static Vec<String> {
    DYNAMIC_COMPLETION_PLUGINS.get_or_init(Vec::new)
}

fn add_plugin_commands_from_manifests(mut cmd: Command) -> Command {
    use lib_plugin_manifest::PluginManifest;

    let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();

    if !plugins_dir.exists() {
        tracing::trace!(dir = %plugins_dir.display(), "Plugins dir does not exist, skipping manifest scan");
        return cmd;
    }

    tracing::trace!(dir = %plugins_dir.display(), "Discovering plugin commands for completions");
    let mut dynamic_plugins = Vec::new();

    for manifest_path in collect_cli_manifest_paths(&plugins_dir) {
        let Ok(manifest) = PluginManifest::from_file(&manifest_path) else {
            continue;
        };
        let Some(cli) = &manifest.cli else { continue };

        let (subcmd, is_dynamic) = build_cli_subcommand(cli);
        if is_dynamic {
            dynamic_plugins.push(cli.command.clone());
        }
        tracing::trace!(command = %cli.command, "Added plugin command to completions");
        cmd = cmd.subcommand(subcmd);
    }

    tracing::trace!(
        dynamic_count = dynamic_plugins.len(),
        "Plugin manifest scan complete"
    );
    let _ = DYNAMIC_COMPLETION_PLUGINS.set(dynamic_plugins);
    cmd
}

fn build_cli_subcommand(cli: &lib_plugin_manifest::CliConfig) -> (Command, bool) {
    let name: &'static str = Box::leak(cli.command.clone().into_boxed_str());
    let desc: &'static str = Box::leak(cli.description.clone().into_boxed_str());
    let mut subcmd = Command::new(name)
        .about(desc)
        .allow_external_subcommands(true);

    for alias in &cli.aliases {
        let alias_static: &'static str = Box::leak(alias.clone().into_boxed_str());
        subcmd = subcmd.visible_alias(alias_static);
    }

    (subcmd, cli.dynamic_completions)
}

fn collect_cli_manifest_paths(plugins_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let cmds_dir = lib_plugin_host::command_index::commands_dir(plugins_dir);

    if cmds_dir.exists() {
        if let Some(indexed) = collect_indexed_manifest_paths(plugins_dir) {
            return indexed;
        }
    }

    tracing::trace!("Command index unavailable, falling back to full scan for completions");
    collect_scanned_manifest_paths(plugins_dir)
}

fn collect_indexed_manifest_paths(plugins_dir: &std::path::Path) -> Option<Vec<std::path::PathBuf>> {
    use std::collections::HashSet;

    let indexed = lib_plugin_host::command_index::list_indexed_commands(plugins_dir);
    if indexed.is_empty() {
        return None;
    }

    tracing::trace!(count = indexed.len(), "Using command index for completions");
    let mut seen = HashSet::new();
    Some(
        indexed
            .into_iter()
            .filter_map(|(_name, path)| seen.insert(path.clone()).then_some(path))
            .collect(),
    )
}

fn collect_scanned_manifest_paths(plugins_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let Ok(entries) = std::fs::read_dir(plugins_dir) else {
        return Vec::new();
    };

    entries
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter(|e| e.file_name() != lib_plugin_host::command_index::COMMANDS_DIR_NAME)
        .filter_map(|e| find_plugin_manifest(&e.path()))
        .collect()
}

fn find_plugin_manifest(plugin_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    crate::plugin_runtime::find_plugin_toml_path(plugin_dir)
}

pub fn get_shell_config_path(shell: CompletionShell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    match shell {
        CompletionShell::Bash => {
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

pub fn get_completions_dir(shell: CompletionShell) -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let data_dir = dirs::data_local_dir().unwrap_or_else(|| home.join(".local/share"));

    match shell {
        CompletionShell::Bash => {
            let xdg = data_dir.join("bash-completion/completions");
            if xdg.parent().map(|p| p.exists()).unwrap_or(false) {
                Some(xdg)
            } else {
                Some(home.join(".bash_completion.d"))
            }
        }
        CompletionShell::Zsh => {
            let zsh_funcs = home.join(".zfunc");
            Some(zsh_funcs)
        }
        CompletionShell::Fish => Some(home.join(".config/fish/completions")),
        CompletionShell::PowerShell => dirs::config_dir().map(|c| c.join("powershell")),
        CompletionShell::Elvish => Some(home.join(".elvish/lib")),
    }
}

pub fn get_completion_filename(shell: CompletionShell, bin_name: &str) -> String {
    match shell {
        CompletionShell::Bash => format!("{}.bash", bin_name),
        CompletionShell::Zsh => format!("_{}", bin_name),
        CompletionShell::Fish => format!("{}.fish", bin_name),
        CompletionShell::PowerShell => format!("_{}.ps1", bin_name),
        CompletionShell::Elvish => format!("{}.elv", bin_name),
    }
}

pub fn init_completions<C: CommandFactory>(
    shell: CompletionShell,
    bin_name: &str,
) -> anyhow::Result<PathBuf> {
    tracing::trace!(shell = ?shell, bin_name = %bin_name, "Initializing shell completions");
    let completions_dir = get_completions_dir(shell)
        .ok_or_else(|| anyhow::anyhow!("Could not determine completions directory"))?;

    std::fs::create_dir_all(&completions_dir)?;

    let completion_file = completions_dir.join(get_completion_filename(shell, bin_name));

    let file = std::fs::File::create(&completion_file)?;
    let mut cmd = C::command();
    cmd = add_plugin_commands_from_manifests(cmd);
    write_completions_to_file(shell, bin_name, &cmd, file)?;

    setup_shell_config(shell, &completion_file)?;

    Ok(completion_file)
}

fn write_completions_to_file(
    shell: CompletionShell,
    bin_name: &str,
    cmd: &Command,
    mut file: std::fs::File,
) -> anyhow::Result<()> {
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

fn generate_zsh_script_with_dynamic(bin_name: &str, cmd: &Command) -> String {
    let dynamic_plugins = get_dynamic_completion_plugins();
    let plugin_commands = build_zsh_plugin_command_entries(cmd);
    let dynamic_cases = build_zsh_dynamic_cases(dynamic_plugins);

    format!(
        "{}\n{}\n",
        zsh_dynamic_complete_fn(bin_name),
        zsh_main_fn(&plugin_commands, &dynamic_cases),
    )
}

fn zsh_dynamic_complete_fn(bin_name: &str) -> String {
    format!(
        r#"#compdef {bin_name}

# Dynamic completion function for plugins
_adi_dynamic_complete() {{
    local cmd=$1
    local pos=$2
    shift 2
    local words=("$@")

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
}}"#
    )
}

fn zsh_main_fn(plugin_commands: &str, dynamic_cases: &str) -> String {
    format!(
        r#"_adi() {{
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
{plugin_commands}            )
            _describe -t commands 'adi commands' commands
            ;;
        args)
            case $line[1] in
{dynamic_cases}                *)
                    _files
                    ;;
            esac
            ;;
    esac
}}

_adi "$@""#
    )
}

const ZSH_BUILTIN_COMMANDS: &[&str] = &[
    "plugin",
    "search",
    "services",
    "run",
    "self-update",
    "completions",
    "info",
];

fn build_zsh_plugin_command_entries(cmd: &Command) -> String {
    let mut entries = String::new();
    for subcmd in cmd.get_subcommands() {
        let name = subcmd.get_name();
        if ZSH_BUILTIN_COMMANDS.contains(&name) {
            continue;
        }
        let about = subcmd
            .get_about()
            .map(|s| s.to_string())
            .unwrap_or_default();
        entries.push_str(&format!("                '{name}:{about}'\n"));
    }
    entries
}

fn build_zsh_dynamic_cases(dynamic_plugins: &[String]) -> String {
    let mut cases = String::new();
    for plugin_cmd in dynamic_plugins {
        cases.push_str(&format!(
            r#"                {plugin_cmd})
                    _adi_dynamic_complete "{plugin_cmd}" $((CURRENT)) "${{words[@]:1}}"
                    ;;
"#
        ));
    }
    cases
}

fn generate_bash_script_with_dynamic(bin_name: &str, cmd: &Command) -> String {
    let dynamic_plugins = get_dynamic_completion_plugins();
    let subcommands: Vec<&str> = cmd.get_subcommands().map(|c| c.get_name()).collect();
    let subcommands_str = subcommands.join(" ");
    let dynamic_str = dynamic_plugins.join("|");

    format!(
        "{}\n{}\ncomplete -F _{bin_name} {bin_name}\n",
        bash_dynamic_complete_fn(bin_name),
        bash_main_fn(bin_name, &subcommands_str, &dynamic_str),
    )
}

fn bash_dynamic_complete_fn(bin_name: &str) -> String {
    format!(
        r#"# Bash completion for {bin_name}

_{bin_name}_dynamic_complete() {{
    local cmd=$1
    local pos=$2
    shift 2
    local words=("$@")

    local completions
    completions=$({bin_name} "$cmd" --completions "$pos" "${{words[@]}}" 2>/dev/null)

    if [[ -n "$completions" ]]; then
        local -a comps
        while IFS=$'\t' read -r comp desc; do
            comps+=("$comp")
        done <<< "$completions"
        COMPREPLY=($(compgen -W "${{comps[*]}}" -- "${{COMP_WORDS[COMP_CWORD]}}"))
        return 0
    fi
    return 1
}}"#
    )
}

fn bash_main_fn(bin_name: &str, subcommands_str: &str, dynamic_str: &str) -> String {
    format!(
        r#"_{bin_name}() {{
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
            local pos=$((cword - 1))
            local cmd_words=("${{words[@]:2}}")
            _{bin_name}_dynamic_complete "$cmd" "$pos" "${{cmd_words[@]}}"
            ;;
        *)
            _filedir
            ;;
    esac
}}"#
    )
}

fn generate_fish_script_with_dynamic(bin_name: &str, cmd: &Command) -> String {
    let dynamic_plugins = get_dynamic_completion_plugins();
    let mut script = fish_header_and_dynamic_fn(bin_name);
    append_fish_subcommand_completions(&mut script, bin_name, cmd);
    script.push('\n');
    append_fish_dynamic_completions(&mut script, bin_name, dynamic_plugins);
    script
}

fn fish_header_and_dynamic_fn(bin_name: &str) -> String {
    format!(
        r#"# Fish completion for {bin_name}

function __adi_dynamic_complete
    set -l cmd $argv[1]
    set -l pos $argv[2]
    set -l words $argv[3..-1]

    set -l completions ({bin_name} $cmd --completions $pos $words 2>/dev/null)

    for line in $completions
        set -l parts (string split \t $line)
        if test (count $parts) -ge 2
            echo $parts[1]\t$parts[2]
        else
            echo $parts[1]
        end
    end
end

complete -c {bin_name} -f
"#
    )
}

fn append_fish_subcommand_completions(script: &mut String, bin_name: &str, cmd: &Command) {
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
}

fn append_fish_dynamic_completions(
    script: &mut String,
    bin_name: &str,
    dynamic_plugins: &[String],
) {
    for plugin_cmd in dynamic_plugins {
        script.push_str(&format!(
            r#"complete -c {bin_name} -n "__fish_seen_subcommand_from {plugin_cmd}" -a "(__adi_dynamic_complete {plugin_cmd} (count (commandline -opc)) (commandline -opc)[3..-1])"
"#
        ));
    }
}

fn add_to_shell_config(shell: CompletionShell, snippet: &str) -> anyhow::Result<()> {
    let config_path = get_shell_config_path(shell)
        .ok_or_else(|| anyhow::anyhow!("Could not determine shell config path"))?;

    let existing = std::fs::read_to_string(&config_path).unwrap_or_default();

    if existing.contains("# ADI CLI completions") {
        return Ok(());
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)?;

    writeln!(file, "{}", snippet)?;

    Ok(())
}

pub fn regenerate_completions<C: CommandFactory>(bin_name: &str) -> anyhow::Result<()> {
    tracing::trace!(bin_name = %bin_name, "Regenerating completions for installed shells");
    for shell in [
        CompletionShell::Bash,
        CompletionShell::Zsh,
        CompletionShell::Fish,
    ] {
        if let Some(dir) = get_completions_dir(shell) {
            let file_path = dir.join(get_completion_filename(shell, bin_name));
            if file_path.exists() {
                tracing::trace!(shell = ?shell, path = %file_path.display(), "Regenerating completion file");
                let file = std::fs::File::create(&file_path)?;
                let mut cmd = C::command();
                cmd = add_plugin_commands_from_manifests(cmd);
                write_completions_to_file(shell, bin_name, &cmd, file)?;
            }
        }
    }

    Ok(())
}

pub fn detect_shell() -> Option<CompletionShell> {
    std::env::var("SHELL").ok().and_then(|s| {
        tracing::trace!(shell_env = %s, "Detecting shell from $SHELL");
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

pub fn ensure_completions_installed<C: CommandFactory>(bin_name: &str) {
    let Some(shell) = detect_shell() else {
        tracing::trace!("Could not detect shell, skipping completions");
        return;
    };

    let Some(completions_dir) = get_completions_dir(shell) else {
        tracing::trace!(shell = ?shell, "Could not determine completions directory");
        return;
    };

    let completion_file = completions_dir.join(get_completion_filename(shell, bin_name));
    let marker_file = completions_dir.join(format!(".{}-installed", bin_name));
    let needs_shell_config = !marker_file.exists();

    if !needs_shell_config && !completions_outdated(&completion_file) {
        tracing::trace!(shell = ?shell, "Completions are up-to-date");
        return;
    }

    tracing::trace!(shell = ?shell, needs_shell_config = needs_shell_config, "Regenerating completions");
    write_completion_file_if_possible::<C>(shell, bin_name, &completions_dir, &completion_file);

    if needs_shell_config {
        let _ = setup_shell_config(shell, &completion_file);
        let _ = std::fs::write(&marker_file, "");
    }
}

fn write_completion_file_if_possible<C: CommandFactory>(
    shell: CompletionShell,
    bin_name: &str,
    completions_dir: &std::path::Path,
    completion_file: &std::path::Path,
) {
    if std::fs::create_dir_all(completions_dir).is_err() {
        return;
    }
    let Ok(file) = std::fs::File::create(completion_file) else {
        return;
    };
    let mut cmd = C::command();
    cmd = add_plugin_commands_from_manifests(cmd);
    let _ = write_completions_to_file(shell, bin_name, &cmd, file);
}

fn completions_outdated(completion_file: &std::path::Path) -> bool {
    let plugins_dir = lib_plugin_host::PluginConfig::default_plugins_dir();

    if !plugins_dir.exists() {
        return false;
    }

    let Ok(completion_meta) = std::fs::metadata(completion_file) else {
        return true;
    };

    let Ok(completion_time) = completion_meta.modified() else {
        return true;
    };

    if let Ok(entries) = std::fs::read_dir(&plugins_dir) {
        for entry in entries.flatten() {
            if entry.file_name() == lib_plugin_host::command_index::COMMANDS_DIR_NAME {
                continue;
            }
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
