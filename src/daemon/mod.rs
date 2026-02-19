//! ADI Daemon - Background process managing plugin services
//!
//! Internal daemon that handles:
//! - Plugin service lifecycle (start, stop, restart)
//! - Health monitoring and auto-restart
//! - Privileged command execution (via adi-root user)
//! - IPC communication with plugins
//!
//! # Architecture
//!
//! The daemon runs as a single background process and manages child services.
//! Communication uses zero-copy rkyv serialization over local sockets.
//!
//! ```text
//! ┌──────────────────────────────────────────┐
//! │               adi daemon                  │
//! ├──────────────────────────────────────────┤
//! │  ServiceMgr   │  IPC Server  │ HealthMgr │
//! │  (children)   │  (socket)    │ (watchdog)│
//! ├──────────────────────────────────────────┤
//! │  hive │ indexer │ proxy │ cocoon │ ...   │
//! └──────────────────────────────────────────┘
//! ```

pub mod client;
pub mod executor;
pub mod health;
pub mod protocol;
pub mod server;
pub mod services;

pub use client::DaemonClient;
pub use executor::CommandExecutor;
pub use health::HealthManager;
pub use protocol::{Request, Response, ServiceConfig, ServiceInfo, ServiceState};
pub use server::DaemonServer;
pub use services::ServiceManager;
