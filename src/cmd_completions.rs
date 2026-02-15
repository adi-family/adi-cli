use cli::completions::{self, CompletionShell};
use lib_console_output::{out_info, out_success};
use lib_i18n_core::t;

use crate::args::Cli;

pub(crate) fn cmd_completions(shell: CompletionShell) {
    completions::generate_completions::<Cli>(shell, "adi");
}

pub(crate) fn cmd_init(shell: Option<CompletionShell>) -> anyhow::Result<()> {
    let shell = shell
        .or_else(completions::detect_shell)
        .ok_or_else(|| anyhow::anyhow!(t!("completions-error-no-shell")))?;

    out_info!("{}", t!("completions-init-start", "shell" => &format!("{:?}", shell)));

    let path = completions::init_completions::<Cli>(shell, "adi")?;

    out_success!("{}", t!("completions-init-done", "path" => &path.display().to_string()));

    match shell {
        CompletionShell::Zsh => out_info!("{}", t!("completions-restart-zsh")),
        CompletionShell::Bash => out_info!("{}", t!("completions-restart-bash")),
        CompletionShell::Fish => out_info!("{}", t!("completions-restart-fish")),
        _ => out_info!("{}", t!("completions-restart-generic")),
    }

    Ok(())
}
