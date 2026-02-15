use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, out_error};
use lib_plugin_abi_v3::logs::LogStreamContext;

pub(crate) async fn cmd_logs(
    plugin_id: &str,
    follow: bool,
    lines: u32,
    level: Option<String>,
    service: Option<String>,
) -> anyhow::Result<()> {
    tracing::trace!(plugin_id = %plugin_id, follow = follow, lines = lines, level = ?level, service = ?service, "cmd_logs invoked");

    let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;

    if let Err(e) = runtime.scan_and_load_plugin(plugin_id).await {
        out_error!("Failed to load plugin {}: {}", plugin_id, e);
        std::process::exit(1);
    }

    tracing::trace!(plugin_id = %plugin_id, "Requesting log provider");
    let log_provider = match runtime.get_log_provider(plugin_id) {
        Some(p) => {
            tracing::trace!("Log provider acquired");
            p
        }
        None => {
            tracing::trace!("Plugin does not provide log streaming");
            out_error!("Plugin {} does not provide log streaming.", theme::brand(plugin_id));
            std::process::exit(1);
        }
    };

    let ctx = LogStreamContext {
        service,
        level,
        tail: Some(lines),
        follow,
    };

    tracing::trace!("Creating log stream");
    let mut stream = log_provider.log_stream(ctx).await.map_err(|e| {
        anyhow::anyhow!("Failed to create log stream: {}", e)
    })?;
    tracing::trace!("Log stream created, reading entries");

    while let Some(line) = stream.next().await {
        let level_colored = match line.level.as_str() {
            "trace" => theme::muted(&line.level).to_string(),
            "debug" => theme::debug(&line.level).to_string(),
            "info" => theme::success(&line.level).to_string(),
            "notice" => theme::debug(&line.level).to_string(),
            "warn" => theme::warning(&line.level).to_string(),
            "error" => theme::error(&line.level).to_string(),
            "fatal" => theme::brand_bold(&line.level).to_string(),
            _ => line.level.clone(),
        };
        let timestamp = line.timestamp.format("%H:%M:%S%.3f");
        println!("{} {} [{}] {}", timestamp, line.service, level_colored, line.message);
    }

    Ok(())
}
