//! HTTP server that dispatches to plugin-provided routes.
//!
//! NOTE: This is a placeholder implementation. Full HTTP server functionality
//! requires thread-safe plugin access which will be addressed in a future update.

use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// HTTP server configuration.
#[derive(Debug, Clone)]
pub struct HttpServerConfig {
    /// Port to listen on.
    pub port: u16,
    /// Host to bind to.
    pub host: String,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            host: "127.0.0.1".to_string(),
        }
    }
}

/// HTTP server that dispatches to plugin-provided routes.
pub struct HttpServer {
    config: HttpServerConfig,
}

impl HttpServer {
    /// Create a new HTTP server.
    pub fn new(config: HttpServerConfig) -> Self {
        Self { config }
    }

    /// Run the HTTP server.
    pub async fn run(&self) -> anyhow::Result<()> {
        // Build the router
        let app = self.build_router();

        // Parse address
        let addr: SocketAddr = format!("{}:{}", self.config.host, self.config.port).parse()?;

        tracing::info!("Starting HTTP server on {}", addr);

        // Start the server
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    fn build_router(&self) -> Router {
        // CORS layer
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        Router::new()
            // Health check
            .route("/health", get(health_handler))
            // Placeholder for plugin routes
            .route("/api/info", get(info_handler))
            .layer(cors)
            .layer(TraceLayer::new_for_http())
    }
}

async fn health_handler() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn info_handler() -> Json<Value> {
    Json(json!({
        "message": "Plugin-provided routes will be available when plugins are loaded",
        "note": "Use 'adi plugin install <id>' to install plugins"
    }))
}
