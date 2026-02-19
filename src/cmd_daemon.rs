use crate::args::DaemonCommands;
use anyhow::Result;
use cli::clienv;
use cli::daemon::server::DaemonConfig;
use cli::daemon::{DaemonClient, DaemonServer};
use lib_console_output::{
    blocks::{KeyValue, Renderable, Section, Table},
    theme,
};

/// Handle daemon commands
pub async fn cmd_daemon(command: DaemonCommands) -> Result<()> {
    match command {
        DaemonCommands::Run => cmd_daemon_run().await,
        DaemonCommands::Start => cmd_daemon_start().await,
        DaemonCommands::Stop { force } => cmd_daemon_stop(force).await,
        DaemonCommands::Restart => cmd_daemon_restart().await,
        DaemonCommands::Status => cmd_daemon_status().await,
        DaemonCommands::StartService { service } => cmd_start_service(&service).await,
        DaemonCommands::StopService { service, force } => cmd_stop_service(&service, force).await,
        DaemonCommands::RestartService { service } => cmd_restart_service(&service).await,
        DaemonCommands::Services => cmd_list_services().await,
        DaemonCommands::Logs {
            service,
            lines,
            follow,
        } => cmd_service_logs(&service, lines, follow).await,
    }
}

/// Run the daemon in foreground (for debugging)
async fn cmd_daemon_run() -> Result<()> {
    println!(
        "{} Running daemon in foreground (Ctrl+C to stop)",
        theme::icons::INFO
    );
    println!(
        "  Socket: {}",
        theme::muted(clienv::daemon_socket_path().display())
    );
    println!(
        "  PID:    {}",
        theme::muted(clienv::daemon_pid_path().display())
    );
    println!();

    let config = DaemonConfig::default();
    let server = DaemonServer::new(config);
    server.run().await
}

/// Start the daemon in background
async fn cmd_daemon_start() -> Result<()> {
    let client = DaemonClient::new();

    if client.is_running().await {
        let (uptime, version) = client.ping().await?;
        println!(
            "{} Daemon already running (v{}, uptime: {})",
            theme::icons::INFO,
            version,
            format_duration(uptime)
        );
        return Ok(());
    }

    println!("{} Starting daemon...", theme::icons::INFO);
    client.ensure_running().await?;

    let (_uptime, version) = client.ping().await?;
    println!(
        "{} Daemon started (v{}, PID written to {})",
        theme::icons::SUCCESS,
        version,
        theme::muted(clienv::daemon_pid_path().display())
    );

    Ok(())
}

/// Stop the daemon
async fn cmd_daemon_stop(force: bool) -> Result<()> {
    let client = DaemonClient::new();

    if !client.is_running().await {
        println!("{} Daemon is not running", theme::icons::INFO);
        return Ok(());
    }

    if force {
        println!("{} Force stopping daemon...", theme::icons::WARNING);
    } else {
        println!("{} Stopping daemon gracefully...", theme::icons::INFO);
    }

    client.shutdown(!force).await?;

    // Wait for daemon to actually stop
    for _ in 0..50 {
        if !client.socket_exists() {
            println!("{} Daemon stopped", theme::icons::SUCCESS);
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    println!(
        "{} Daemon may still be stopping (check with `adi daemon status`)",
        theme::icons::WARNING
    );
    Ok(())
}

/// Restart the daemon
async fn cmd_daemon_restart() -> Result<()> {
    println!("{} Restarting daemon...", theme::icons::INFO);
    cmd_daemon_stop(false).await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    cmd_daemon_start().await
}

/// Show daemon and services status
async fn cmd_daemon_status() -> Result<()> {
    let client = DaemonClient::new();

    Section::new("Daemon Status").print();

    if !client.socket_exists() {
        println!(
            "\n  {} Status: {}",
            theme::icons::ERROR,
            theme::error("not running")
        );
        println!(
            "  {} Run `adi daemon start` to start the daemon\n",
            theme::icons::INFO
        );
        return Ok(());
    }

    match client.ping().await {
        Ok((uptime, version)) => {
            println!();
            KeyValue::new()
                .entry("Status", theme::success("running").to_string())
                .entry("Version", version)
                .entry("Uptime", format_duration(uptime))
                .entry("Socket", clienv::daemon_socket_path().display().to_string())
                .entry("PID File", clienv::daemon_pid_path().display().to_string())
                .entry("Log File", clienv::daemon_log_path().display().to_string())
                .print();
            println!();

            // Show services
            let services = client.list_services().await?;
            if !services.is_empty() {
                Section::new("Managed Services").print();
                println!();

                let mut table = Table::new().header(["Service", "State", "PID", "Uptime", "Restarts"]);

                for svc in &services {
                    let state_str = format_state(svc.state.as_str());
                    let pid_str = svc
                        .pid
                        .map(|p| p.to_string())
                        .unwrap_or_else(|| "-".to_string());
                    let uptime_str = svc
                        .uptime_secs
                        .map(format_duration)
                        .unwrap_or_else(|| "-".to_string());

                    table = table.row([
                        svc.name.clone(),
                        state_str,
                        pid_str,
                        uptime_str,
                        svc.restarts.to_string(),
                    ]);
                }

                table.print();
                println!();
            } else {
                println!(
                    "  {} No services currently managed\n",
                    theme::icons::INFO
                );
            }
        }
        Err(e) => {
            println!(
                "\n  {} Status: {} (socket exists but not responding)",
                theme::icons::WARNING,
                theme::warning("unhealthy")
            );
            println!("  {} Error: {}\n", theme::icons::ERROR, e);
        }
    }

    Ok(())
}

/// Start a managed service
async fn cmd_start_service(name: &str) -> Result<()> {
    let client = DaemonClient::new();
    client.ensure_running().await?;

    println!(
        "{} Starting service {}...",
        theme::icons::INFO,
        theme::bold(name)
    );
    client.start_service(name, None).await?;
    println!(
        "{} Service {} started",
        theme::icons::SUCCESS,
        theme::bold(name)
    );

    Ok(())
}

/// Stop a managed service
async fn cmd_stop_service(name: &str, force: bool) -> Result<()> {
    let client = DaemonClient::new();

    if !client.is_running().await {
        anyhow::bail!("Daemon is not running. Start it with `adi daemon start`");
    }

    if force {
        println!(
            "{} Force stopping service {}...",
            theme::icons::WARNING,
            theme::bold(name)
        );
    } else {
        println!(
            "{} Stopping service {}...",
            theme::icons::INFO,
            theme::bold(name)
        );
    }

    client.stop_service(name, force).await?;
    println!(
        "{} Service {} stopped",
        theme::icons::SUCCESS,
        theme::bold(name)
    );

    Ok(())
}

/// Restart a managed service
async fn cmd_restart_service(name: &str) -> Result<()> {
    let client = DaemonClient::new();
    client.ensure_running().await?;

    println!(
        "{} Restarting service {}...",
        theme::icons::INFO,
        theme::bold(name)
    );
    client.restart_service(name).await?;
    println!(
        "{} Service {} restarted",
        theme::icons::SUCCESS,
        theme::bold(name)
    );

    Ok(())
}

/// List all managed services
async fn cmd_list_services() -> Result<()> {
    let client = DaemonClient::new();

    if !client.is_running().await {
        anyhow::bail!("Daemon is not running. Start it with `adi daemon start`");
    }

    let services = client.list_services().await?;

    if services.is_empty() {
        println!("{} No services registered", theme::icons::INFO);
        println!(
            "  Services are registered when plugins with service definitions are loaded."
        );
        return Ok(());
    }

    Section::new("Services").print();
    println!();

    let mut table = Table::new().header(["Service", "State", "PID", "Uptime", "Restarts"]);

    for svc in &services {
        let state_str = format_state(svc.state.as_str());
        let pid_str = svc
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());
        let uptime_str = svc
            .uptime_secs
            .map(format_duration)
            .unwrap_or_else(|| "-".to_string());

        table = table.row([
            svc.name.clone(),
            state_str,
            pid_str,
            uptime_str,
            svc.restarts.to_string(),
        ]);
    }

    table.print();
    println!();

    Ok(())
}

/// View service logs
async fn cmd_service_logs(name: &str, lines: usize, follow: bool) -> Result<()> {
    let client = DaemonClient::new();

    if !client.is_running().await {
        anyhow::bail!("Daemon is not running. Start it with `adi daemon start`");
    }

    if follow {
        println!(
            "{} Streaming logs for {} (Ctrl+C to stop)...",
            theme::icons::INFO,
            theme::bold(name)
        );
        // TODO: Implement log streaming
        println!(
            "{} Log streaming not yet implemented",
            theme::icons::WARNING
        );
    } else {
        let logs = client.service_logs(name, lines).await?;

        if logs.is_empty() {
            println!("{} No logs available for {}", theme::icons::INFO, name);
        } else {
            Section::new(&format!("Logs: {}", name)).print();
            println!();
            for line in logs {
                println!("  {}", line);
            }
            println!();
        }
    }

    Ok(())
}

/// Format state with colors
fn format_state(state: &str) -> String {
    match state {
        "running" => theme::success("running").to_string(),
        "starting" => theme::info("starting").to_string(),
        "stopping" => theme::warning("stopping").to_string(),
        "stopped" => theme::muted("stopped").to_string(),
        "failed" => theme::error("failed").to_string(),
        other => other.to_string(),
    }
}

/// Format duration in human-readable form
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}
