# ADI Daemon Architecture

Background daemon process that manages plugin services. Power users can control the daemon directly via `adi daemon` commands.

## Quick Start

```bash
# Check daemon status
adi daemon status

# Start/stop the daemon
adi daemon start
adi daemon stop

# Manage services
adi daemon services        # List all services
adi daemon start hive      # Start a service
adi daemon stop hive       # Stop a service
adi daemon restart hive    # Restart a service

# Debug: run daemon in foreground
adi daemon run
```

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                         adi daemon                                │
│                    (single background process)                    │
├──────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
│  │ ServiceMgr  │  │  IPC Server │  │  HealthMgr  │              │
│  │ (children)  │  │  (socket)   │  │  (watchdog) │              │
│  └──────┬──────┘  └─────────────┘  └─────────────┘              │
│         │                                                         │
│  ┌──────┴─────────────────────────────────────────────┐          │
│  │              Managed Services (children)            │          │
│  │  ┌───────┐  ┌─────────┐  ┌───────┐  ┌─────────┐   │          │
│  │  │ hive  │  │ indexer │  │ proxy │  │ cocoon  │   │          │
│  │  └───────┘  └─────────┘  └───────┘  └─────────┘   │          │
│  └─────────────────────────────────────────────────────┘          │
│                                                                    │
│  Socket: ~/.local/share/adi/daemon.sock                           │
│  PID:    ~/.local/share/adi/daemon.pid                            │
│  Logs:   ~/.local/share/adi/logs/daemon.log                       │
└──────────────────────────────────────────────────────────────────┘
```

## Privilege Model

Two system users provide privilege isolation:

| User | Sudo Access | Purpose |
|------|-------------|---------|
| `adi` | No | Regular commands, plugins run as this user |
| `adi-root` | Yes (NOPASSWD) | Privileged commands only |

### How It Works

1. Daemon runs as `adi` user
2. Plugins run as `adi` user - cannot escalate privileges directly
3. Plugin requests privileged command via IPC: `daemon.sudo_run("iptables", [...])`
4. Daemon executes as `adi-root` user which has sudo access
5. Result returns to plugin

```
┌────────────────────────────┐         ┌──────────────────────────┐
│      Plugin (adi user)     │         │        Daemon            │
│                            │         │                          │
│  run("ls", ["-la"])        │ ──IPC─► │  Execute as `adi`        │
│                            │ ◄────── │  Return result           │
│                            │         │                          │
│  sudo_run("iptables", ..)  │ ──IPC─► │  Execute as `adi-root`   │
│                            │ ◄────── │  Return result           │
└────────────────────────────┘         └──────────────────────────┘
```

### Security Guarantees

- Plugin cannot bypass daemon - `adi` user has no sudo rights
- Plugin cannot trick daemon - `sudo_run()` explicitly uses `adi-root`
- Even if plugin runs `run("sudo", ["..."])`, it fails - `adi` cannot sudo
- Daemon controls all privilege escalation decisions

### Setup (one-time)

```bash
# Create system users
sudo useradd -r -s /bin/false adi
sudo useradd -r -s /bin/false adi-root

# Grant adi-root passwordless sudo
echo "adi-root ALL=(root) NOPASSWD: ALL" | sudo tee /etc/sudoers.d/adi-root
sudo chmod 440 /etc/sudoers.d/adi-root
```

### Plugin Manifest

Plugins declare required privileged commands:

```toml
[package.metadata.plugin.service]
name = "network-manager"
command = "serve"
privileged_commands = [
    "iptables -t nat *",
    "pfctl -f *",
]
```

On install, user approves:

```bash
adi plugin install network-tools
> This plugin requests root access for:
>   - iptables -t nat *
>   - pfctl -f *
> Allow? [y/N]
```

## File Structure

```
crates/cli/src/
├── daemon/
│   ├── mod.rs              # Module exports
│   ├── protocol.rs         # IPC request/response types
│   ├── server.rs           # Daemon main loop + IPC handler
│   ├── services.rs         # Child process management
│   ├── health.rs           # Health checks + watchdog
│   └── client.rs           # Client API for plugins
│
└── (modified)
    ├── main.rs             # Auto-start daemon when needed
    └── clienv.rs           # Daemon env vars
```

## IPC Stack (Decision)

> **This is a deliberate architectural decision, not AI-generated suggestions.**

We need the fastest possible cross-platform IPC. After evaluating options:

| Option | Verdict |
|--------|---------|
| gRPC | Heavy dependency, HTTP/2 overhead, ~100μs latency. Overkill. |
| TCP localhost | Port conflicts, any process can connect, slower than local sockets. |
| JSON over Unix sockets | Good but JSON parsing adds ~500ns per message. |
| D-Bus | Linux-only. Non-starter. |
| Shared memory | Fast but complex, no streaming support. |

### Chosen Stack

```
┌─────────────────────────────────────┐
│       rkyv (zero-copy serde)        │  ← 0ns deserialize, ~50ns serialize
├─────────────────────────────────────┤
│         interprocess crate          │  ← single API, fastest native per OS
├───────────┬───────────┬─────────────┤
│  macOS    │  Linux    │  Windows    │
│  kqueue   │  epoll    │  IOCP       │
│  Unix     │  Unix     │  Named      │
│  socket   │  socket   │  Pipes      │
└───────────┴───────────┴─────────────┘
```

### Why This Stack

1. **`rkyv`** - Zero-copy deserialization. The bytes ARE the struct. No parsing.
2. **`interprocess`** - Cross-platform local sockets using fastest native primitive per OS.
3. **`tokio`** - Async runtime auto-selects kqueue (macOS) / epoll (Linux) / IOCP (Windows).

### Performance

| Serialization | Deserialize | Serialize |
|---------------|-------------|-----------|
| **rkyv** | 0 ns (zero-copy) | ~50 ns |
| bincode | ~100 ns | ~80 ns |
| JSON | ~800 ns | ~500 ns |

### Dependencies

```toml
[dependencies]
rkyv = { version = "0.8", features = ["validation"] }
interprocess = "2"
tokio = { version = "1", features = ["full"] }
```

### Socket Paths

| Platform | Path |
|----------|------|
| macOS/Linux | `~/.local/share/adi/daemon.sock` |
| Linux (abstract) | `@adi-daemon` (no filesystem) |
| Windows | `\\.\pipe\adi-daemon` |

---

## IPC Protocol

```rust
// daemon/protocol.rs

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Archive, Deserialize, Serialize)]
pub enum Request {
    // Daemon lifecycle
    Ping,
    Shutdown { graceful: bool },
    
    // Service management
    StartService { name: String, config: Option<ServiceConfig> },
    StopService { name: String, force: bool },
    RestartService { name: String },
    ListServices,
    ServiceLogs { name: String, lines: usize, follow: bool },
    
    // Command execution
    Run { command: String, args: Vec<String> },
    SudoRun { command: String, args: Vec<String>, reason: String },
}

#[derive(Archive, Deserialize, Serialize)]
pub enum Response {
    Pong { uptime_secs: u64, version: String },
    Ok,
    Error { message: String },
    Services { list: Vec<ServiceInfo> },
    Logs { lines: Vec<String> },
    // Streaming
    LogLine { line: String },
    StreamEnd,
    // Command execution
    CommandResult { exit_code: i32, stdout: Vec<u8>, stderr: Vec<u8> },
    SudoDenied { reason: String },
}

#[derive(Archive, Deserialize, Serialize)]
pub struct ServiceInfo {
    pub name: String,
    pub state: ServiceState,
    pub pid: Option<u32>,
    pub uptime_secs: Option<u64>,
    pub restarts: u32,
    pub last_error: Option<String>,
}

#[derive(Archive, Deserialize, Serialize)]
pub enum ServiceState {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}

#[derive(Archive, Deserialize, Serialize)]
pub struct ServiceConfig {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub working_dir: Option<PathBuf>,
    pub restart_on_failure: bool,
    pub max_restarts: u32,
    pub privileged: bool,  // Run as adi-root instead of adi
}
```

### Zero-Copy Usage

```rust
use rkyv::rancor::Error;
use interprocess::local_socket::{prelude::*, GenericFilePath, Stream};

// Send request
let request = Request::Ping;
let bytes = rkyv::to_bytes::<Error>(&request)?;
stream.write_all(&(bytes.len() as u32).to_le_bytes())?;
stream.write_all(&bytes)?;

// Receive response - ZERO COPY
let mut len_buf = [0u8; 4];
stream.read_exact(&mut len_buf)?;
let len = u32::from_le_bytes(len_buf) as usize;

let mut buf = vec![0u8; len];
stream.read_exact(&mut buf)?;

// No parsing - direct memory access
let archived = rkyv::access::<ArchivedResponse, Error>(&buf)?;
match archived {
    ArchivedResponse::Pong { uptime_secs, version } => {
        // uptime_secs and version are usable directly, no copy
    }
    // ...
}
```

## Command Execution

```rust
// daemon/executor.rs

pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute command as `adi` user (regular)
    pub async fn run(&self, cmd: &str, args: &[String]) -> Result<Output> {
        Command::new("sudo")
            .args(["-u", "adi", cmd])
            .args(args)
            .output()
            .await
    }
    
    /// Execute command as `adi-root` user (privileged)
    pub async fn sudo_run(&self, cmd: &str, args: &[String]) -> Result<Output> {
        Command::new("sudo")
            .args(["-u", "adi-root", "sudo", cmd])
            .args(args)
            .output()
            .await
    }
}
```

## Daemon Server

```rust
// daemon/server.rs

pub struct DaemonServer {
    config: DaemonConfig,
    services: ServiceManager,
    health: HealthManager,
    shutdown: ShutdownCoordinator,
    executor: CommandExecutor,
}

impl DaemonServer {
    pub async fn run(config: DaemonConfig) -> Result<()> {
        // Write PID file
        let mut pid_file = PidFile::new(&config.pid_path);
        pid_file.write()?;

        // Bind IPC socket
        let server = IpcServer::bind(IpcEndpoint::for_path(&config.socket_path)).await?;

        // Start auto-start services
        let mut services = ServiceManager::new();
        for name in &config.auto_start {
            services.start(name).await?;
        }

        // Health check loop
        let health = HealthManager::new(services.clone());
        tokio::spawn(health.run());

        // Main loop
        let mut shutdown = ShutdownCoordinator::new();
        loop {
            tokio::select! {
                conn = server.accept() => {
                    let services = services.clone();
                    tokio::spawn(handle_connection(conn?, services));
                }
                _ = shutdown.wait() => {
                    // Graceful shutdown
                    services.stop_all().await;
                    break;
                }
            }
        }

        Ok(())
    }
}

async fn handle_connection(stream: IpcStream, services: ServiceManager) {
    // Read request, dispatch, send response
}
```

## Service Manager

```rust
// daemon/services.rs

pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, ManagedService>>>,
    registry: ServiceRegistry,
}

pub struct ManagedService {
    pub config: ServiceConfig,
    pub state: ServiceState,
    pub process: Option<Child>,
    pub started_at: Option<Instant>,
    pub restarts: u32,
    pub last_error: Option<String>,
}

impl ServiceManager {
    pub async fn start(&self, name: &str) -> Result<()>;
    pub async fn stop(&self, name: &str, force: bool) -> Result<()>;
    pub async fn restart(&self, name: &str) -> Result<()>;
    pub fn list(&self) -> Vec<ServiceInfo>;
    pub fn get(&self, name: &str) -> Option<ServiceInfo>;
    pub async fn stop_all(&self);
}

// Registry knows how to start each service type
pub struct ServiceRegistry {
    // Built-in services
    hive: HiveService,
    indexer: IndexerService,
    proxy: ProxyService,
    // Plugin-provided services discovered at runtime
    plugins: HashMap<String, PluginService>,
}
```

## Health Manager

```rust
// daemon/health.rs

pub struct HealthManager {
    services: ServiceManager,
    check_interval: Duration,
}

impl HealthManager {
    pub async fn run(&self) {
        let mut interval = tokio::time::interval(self.check_interval);
        
        loop {
            interval.tick().await;
            
            for service in self.services.list() {
                if service.state == ServiceState::Running {
                    if !self.is_healthy(&service).await {
                        // Process died unexpectedly
                        if service.config.restart_on_failure 
                           && service.restarts < service.config.max_restarts {
                            self.services.restart(&service.name).await.ok();
                        } else {
                            self.services.mark_failed(&service.name, "process exited").await;
                        }
                    }
                }
            }
        }
    }

    async fn is_healthy(&self, service: &ServiceInfo) -> bool {
        // Check if process still running
        if let Some(pid) = service.pid {
            is_process_running(pid)
        } else {
            false
        }
    }
}
```

## Daemon Client

```rust
// daemon/client.rs

pub struct DaemonClient {
    socket_path: PathBuf,
}

impl DaemonClient {
    pub fn new() -> Self {
        Self {
            socket_path: default_socket_path(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.socket_path.exists()
    }

    pub async fn ping(&self) -> Result<PongInfo> {
        self.request(&Request::Ping).await
    }

    pub async fn shutdown(&self, graceful: bool) -> Result<()> {
        self.request(&Request::Shutdown { graceful }).await
    }

    pub async fn start_service(&self, name: &str) -> Result<()> {
        self.request(&Request::StartService { 
            name: name.to_string(), 
            config: None 
        }).await
    }

    pub async fn stop_service(&self, name: &str, force: bool) -> Result<()> {
        self.request(&Request::StopService { 
            name: name.to_string(), 
            force 
        }).await
    }

    pub async fn list_services(&self) -> Result<Vec<ServiceInfo>> {
        match self.request(&Request::ListServices).await? {
            Response::Services { list } => Ok(list),
            _ => Err(anyhow!("unexpected response")),
        }
    }

    pub async fn ensure_running(&self) -> Result<()> {
        if !self.is_running() {
            start_daemon()?;
            // Wait for socket to appear
            for _ in 0..50 {
                if self.socket_path.exists() {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            return Err(anyhow!("daemon failed to start"));
        }
        Ok(())
    }

    async fn request<R: DeserializeOwned>(&self, req: &Request) -> Result<R> {
        let client = IpcClient::for_path(&self.socket_path);
        client.request(req).await
    }
}

fn start_daemon() -> Result<()> {
    let exe = std::env::current_exe()?;
    spawn_background(&SpawnConfig::new(exe.display().to_string())
        .args(["daemon", "run"])
        .stdout(log_path().display().to_string())
        .stderr(log_path().display().to_string())
        .pid_file(pid_path().display().to_string())
    )?;
    Ok(())
}
```

## Plugin Integration

Plugins register as daemon-managed services via Cargo.toml metadata:

```toml
# Cargo.toml
[package.metadata.plugin.service]
name = "hive"
command = "serve"  # Plugin subcommand to run as service
restart_on_failure = true
max_restarts = 3
```

The daemon automatically:
- Discovers installed plugins with `[service]` configuration
- Starts services on-demand when plugins request them
- Monitors health and restarts failed services
- Provides IPC for plugins to query service status

### Plugin Client Usage

```rust
use adi_daemon::DaemonClient;

// Plugins use DaemonClient to interact with the daemon
let client = DaemonClient::new();

// Ensure daemon is running (auto-starts if needed)
client.ensure_running().await?;

// Start this plugin's service
client.start_service("hive").await?;

// Query service status
let services = client.list_services().await?;

// Execute regular command (runs as `adi` user)
let output = client.run("ls", &["-la"]).await?;

// Execute privileged command (runs as `adi-root` user)
let output = client.sudo_run("iptables", &["-L"], "List firewall rules").await?;
```

## Internal Configuration

Daemon reads service configs from plugin manifests. Internal tuning via environment:

| Variable | Default | Description |
|----------|---------|-------------|
| `ADI_DAEMON_SOCKET` | `~/.local/share/adi/daemon.sock` | IPC socket path |
| `ADI_DAEMON_PID` | `~/.local/share/adi/daemon.pid` | PID file path |
| `ADI_DAEMON_LOG` | `~/.local/share/adi/logs/daemon.log` | Log file path |
| `ADI_USER` | `adi` | Regular execution user |
| `ADI_ROOT_USER` | `adi-root` | Privileged execution user |
