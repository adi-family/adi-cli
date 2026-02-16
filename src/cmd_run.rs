use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, blocks::{Columns, Section, Renderable}, out_info, out_error};
use lib_i18n_core::{t, LocalizedError};

pub(crate) async fn cmd_run(plugin_id: Option<String>, args: Vec<String>) -> anyhow::Result<()> {
    tracing::trace!(plugin_id = ?plugin_id, args = ?args, "cmd_run invoked");

    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
    runtime.load_all_plugins().await?;

    let runnable = runtime.list_runnable_plugins();
    tracing::trace!(runnable_count = runnable.len(), "Loaded runnable plugins");

    let plugin_id = match plugin_id {
        Some(id) => id,
        None => {
            Section::new(t!("run-title")).print();

            if runnable.is_empty() {
                out_info!("{}", t!("run-empty"));
                out_info!("{}", t!("run-hint-install"));
            } else {
                Columns::new()
                    .header(["Plugin", "Description"])
                    .rows(runnable.iter().map(|(id, desc)| [
                        theme::brand_bold(id).to_string(),
                        theme::muted(desc).to_string(),
                    ]))
                    .print();
                out_info!("{}", t!("run-hint-usage"));
            }
            return Ok(());
        }
    };

    if !runnable.iter().any(|(id, _)| id == &plugin_id) {
        out_error!("{} {}", t!("common-error-prefix"), t!("run-error-not-found", "id" => &plugin_id));
        if runnable.is_empty() {
            out_error!("{}", t!("run-error-no-plugins"));
        } else {
            out_info!("{}", t!("run-error-available"));
            for (id, _) in &runnable {
                out_info!("  - {}", id);
            }
        }
        std::process::exit(1);
    }

    let context = serde_json::json!({
        "command": plugin_id,
        "args": args,
        "cwd": std::env::current_dir()?.to_string_lossy()
    });

    match runtime.run_cli_command(&plugin_id, &context.to_string()).await {
        Ok(result) => {
            handle_cli_result(&result);
            Ok(())
        }
        Err(e) => {
            out_error!("{} {}", t!("common-error-prefix"), t!("run-error-failed", "error" => &e.localized()));
            std::process::exit(1);
        }
    }
}

pub(crate) fn handle_cli_result(result_json: &str) {
    #[derive(serde::Deserialize)]
    struct CliResult {
        exit_code: i32,
        stdout: String,
        stderr: String,
    }

    match serde_json::from_str::<CliResult>(result_json) {
        Ok(result) => {
            tracing::trace!(exit_code = result.exit_code, stdout_len = result.stdout.len(), stderr_len = result.stderr.len(), "Handling CLI result");
            if !result.stdout.is_empty() {
                print!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            if result.exit_code != 0 {
                std::process::exit(result.exit_code);
            }
        }
        Err(e) => {
            tracing::trace!(error = %e, "Failed to parse CLI result JSON, printing raw");
            println!("{}", result_json);
        }
    }
}
