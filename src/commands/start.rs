use cli::plugin_registry::PluginManager;
use cli::plugin_runtime::{PluginRuntime, RuntimeConfig};
use lib_console_output::{theme, blocks::{KeyValue, Renderable}, out_info, out_success};
use std::sync::Arc;

pub(crate) async fn cmd_start(port: u16) -> anyhow::Result<()> {
    use axum::{routing::{get, post}, Router};
    use tower_http::cors::{Any, CorsLayer};
    use tokio::sync::RwLock;

    out_info!("{}", theme::brand_bold("Starting ADI local server..."));

    // Ensure cocoon plugin is installed
    let manager = PluginManager::new();
    let installed = manager.list_installed().await?;
    let cocoon_installed = installed.iter().any(|(id, _)| id == "adi.cocoon");

    if !cocoon_installed {
        out_info!("{}", theme::muted("Installing cocoon plugin..."));
        manager.install_plugin("adi.cocoon", None).await?;
        out_success!("Cocoon plugin installed!");
    }

    let hostname = get_machine_name();

    let (connect_tx, mut connect_rx) = tokio::sync::mpsc::channel::<ConnectRequest>(1);

    let state = Arc::new(StartServerState {
        connected: RwLock::new(false),
        hostname: hostname.clone(),
        connect_tx,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/connect", post(connect_handler))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    // Detect capabilities
    let capabilities = detect_capabilities();
    let ai_agents: Vec<_> = capabilities.iter()
        .filter(|c| c.category == "ai-agent")
        .map(|c| c.name)
        .collect();
    let runtimes: Vec<_> = capabilities.iter()
        .filter(|c| c.category == "runtime")
        .map(|c| c.name)
        .collect();

    let mut kv = KeyValue::new()
        .entry("Name", theme::bold(&hostname).to_string())
        .entry("URL", theme::brand(format!("http://localhost:{}", port)).to_string());
    if !ai_agents.is_empty() {
        kv = kv.entry("Agents", theme::success(ai_agents.join(", ")).to_string());
    }
    if !runtimes.is_empty() {
        kv = kv.entry("Runtimes", theme::info(runtimes.join(", ")).to_string());
    }
    kv.print();

    out_info!("{}", theme::muted("Waiting for browser connection... (Ctrl+C to stop)"));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await
    });

    if let Some(req) = connect_rx.recv().await {
        out_success!("Browser connected! Starting cocoon...");

        std::env::set_var("SIGNALING_SERVER_URL", &req.signaling_url);
        std::env::set_var("COCOON_SETUP_TOKEN", &req.token);
        std::env::set_var("COCOON_NAME", &hostname);

        let runtime = PluginRuntime::new(RuntimeConfig::default()).await?;
        runtime.scan_and_load_plugin("adi.cocoon").await?;

        let install_context = serde_json::json!({
            "command": "adi.cocoon",
            "args": ["create", "--runtime", "machine", "--start"],
            "cwd": std::env::current_dir().unwrap_or_default().to_string_lossy()
        });

        runtime.run_cli_command("adi.cocoon", &install_context.to_string()).await?;

        out_success!("Cocoon installed and running as a background service!");
        KeyValue::new()
            .entry("Status", theme::brand("adi cocoon status").to_string())
            .entry("Logs", theme::brand("adi cocoon logs").to_string())
            .entry("Stop", theme::brand("adi cocoon stop").to_string())
            .print();
    }

    server.abort();

    Ok(())
}

/// Shared state for the start server
struct StartServerState {
    connected: tokio::sync::RwLock<bool>,
    hostname: String,
    connect_tx: tokio::sync::mpsc::Sender<ConnectRequest>,
}

/// Request body for connect endpoint
#[derive(serde::Deserialize)]
struct ConnectRequest {
    token: String,
    #[serde(default = "default_signaling_url")]
    signaling_url: String,
}

fn default_signaling_url() -> String {
    cli::clienv::signaling_url()
}

/// Health endpoint handler for browser polling
async fn health_handler(
    axum::extract::State(state): axum::extract::State<Arc<StartServerState>>,
) -> axum::Json<serde_json::Value> {
    let connected = *state.connected.read().await;

    axum::Json(serde_json::json!({
        "status": "ok",
        "name": state.hostname,
        "version": env!("CARGO_PKG_VERSION"),
        "connected": connected
    }))
}

/// Connect endpoint - browser sends token to register with platform
async fn connect_handler(
    axum::extract::State(state): axum::extract::State<Arc<StartServerState>>,
    axum::Json(req): axum::Json<ConnectRequest>,
) -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    if *state.connected.read().await {
        return (
            axum::http::StatusCode::OK,
            axum::Json(serde_json::json!({
                "status": "already_connected",
                "name": state.hostname
            })),
        );
    }

    if let Err(e) = state.connect_tx.send(req).await {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "status": "error",
                "message": format!("Failed to process request: {}", e)
            })),
        );
    }

    *state.connected.write().await = true;

    (
        axum::http::StatusCode::OK,
        axum::Json(serde_json::json!({
            "status": "connecting",
            "name": state.hostname
        })),
    )
}

/// Get a friendly machine name for display
fn get_machine_name() -> String {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("scutil").args(["--get", "ComputerName"]).output() {
            if output.status.success() {
                if let Ok(name) = String::from_utf8(output.stdout) {
                    let name = name.trim();
                    if !name.is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
    }

    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Local Machine".to_string())
}

/// Detected capability on the machine
struct Capability {
    name: &'static str,
    category: &'static str,
}

/// Detect available tools/capabilities on the machine
fn detect_capabilities() -> Vec<Capability> {
    use std::process::Command;

    let tools: &[(&str, &str)] = &[
        // AI Coding Agents
        ("claude", "ai-agent"),
        ("codex", "ai-agent"),
        ("aider", "ai-agent"),
        ("cursor", "ai-agent"),
        ("copilot", "ai-agent"),
        ("gemini", "ai-agent"),
        // Languages & Runtimes
        ("node", "runtime"),
        ("bun", "runtime"),
        ("deno", "runtime"),
        ("python3", "runtime"),
        ("python", "runtime"),
        ("cargo", "runtime"),
        ("go", "runtime"),
        ("java", "runtime"),
        ("ruby", "runtime"),
        ("php", "runtime"),
        ("dotnet", "runtime"),
        ("swift", "runtime"),
        // Dev Tools
        ("git", "tool"),
        ("gh", "tool"),
        ("docker", "tool"),
        ("kubectl", "tool"),
        ("terraform", "tool"),
        ("aws", "tool"),
        ("gcloud", "tool"),
        ("az", "tool"),
    ];

    let mut capabilities = Vec::new();

    for (cmd, category) in tools {
        let result = if cfg!(windows) {
            Command::new("where").arg(cmd).output()
        } else {
            Command::new("which").arg(cmd).output()
        };

        if let Ok(output) = result {
            if output.status.success() {
                capabilities.push(Capability { name: cmd, category });
            }
        }
    }

    capabilities
}
