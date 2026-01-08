// Caret Inspector - HTTP/WebSocket server
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{api::{ApiState, InspectorApi}, Inspector, InspectorHandle};
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

// Re-export serde_json for use in other modules
pub use serde_json;

/// Configuration for the inspector server
#[derive(Clone, Debug)]
pub struct InspectorConfig {
    /// Bind address
    pub bind_addr: SocketAddr,
    /// Enable CORS
    pub enable_cors: bool,
    /// Event stream buffer capacity
    pub event_buffer_capacity: usize,
}

impl Default for InspectorConfig {
    fn default() -> Self {
        Self {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            enable_cors: true,
            event_buffer_capacity: 100,
        }
    }
}

impl InspectorConfig {
    /// Create a new inspector configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the bind address
    pub fn with_bind_addr(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.bind_addr = addr.into();
        self
    }

    /// Enable or disable CORS
    pub fn with_cors(mut self, enable: bool) -> Self {
        self.enable_cors = enable;
        self
    }

    /// Set the event buffer capacity
    pub fn with_event_buffer_capacity(mut self, capacity: usize) -> Self {
        self.event_buffer_capacity = capacity;
        self
    }
}

/// Inspector server
///
/// Runs an HTTP server with REST API and WebSocket endpoints
/// for runtime introspection.
pub struct InspectorServer {
    /// Inspector
    inspector: Inspector,
    /// Event stream
    events: crate::EventStream,
    /// Server configuration
    config: InspectorConfig,
}

impl InspectorServer {
    /// Create a new inspector server
    pub fn new(config: InspectorConfig) -> Self {
        let inspector = Inspector::new();
        let events = crate::EventStream::new(config.event_buffer_capacity);

        Self {
            inspector,
            events,
            config,
        }
    }

    /// Create a new inspector server with default configuration
    pub fn with_default_config() -> Self {
        Self::new(InspectorConfig::default())
    }

    /// Get the inspector
    pub fn inspector(&self) -> &Inspector {
        &self.inspector
    }

    /// Get the event stream
    pub fn events(&self) -> &crate::EventStream {
        &self.events
    }

    /// Build the router
    fn build_router(&self) -> Router {
        let state = ApiState {
            inspector: InspectorHandle::new(self.inspector.clone()),
            events: self.events.clone(),
        };

        let mut router = InspectorApi::new(state).router();

        // Add WebSocket endpoint
        let state = AppState {
            events: self.events.clone(),
        };
        router = Router::new()
            .route("/api/stream", get(ws_handler))
            .with_state(state)
            .merge(router);

        // Add CORS if enabled
        if self.config.enable_cors {
            router = router.layer(CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any));
        }

        router
    }

    /// Start the server
    ///
    /// This function runs the server and blocks until the server is shut down.
    pub async fn run(self) -> Result<(), ServerError> {
        let app = self.build_router();
        let addr = self.config.bind_addr;

        tracing::info!("Inspector server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::BindFailed(e.to_string()))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| ServerError::ServeError(e.to_string()))
    }

    /// Start the server in the background
    ///
    /// Returns a handle that can be used to stop the server.
    pub async fn spawn(self) -> Result<ServerHandle, ServerError> {
        let app = self.build_router();
        let addr = self.config.bind_addr;

        tracing::info!("Inspector server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .map_err(|e| ServerError::BindFailed(e.to_string()))?;

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        let handle = tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    // Wait for shutdown signal
                    let _ = shutdown_rx.await;
                    tracing::info!("Inspector server shutting down");
                })
                .await
                .map_err(|e| ServerError::ServeError(e.to_string()))
        });

        Ok(ServerHandle {
            shutdown: Some(shutdown_tx),
            handle: Some(handle),
        })
    }
}

/// Application state for WebSocket handler
#[derive(Clone)]
struct AppState {
    /// Event stream
    events: crate::EventStream,
}

/// WebSocket handler for /api/stream
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handle a WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to events
    let mut rx = state.events.subscribe();

    // Spawn a task to send events
    let send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            let json = match serde_json::to_string(&event) {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!("Failed to serialize event: {}", e);
                    continue;
                }
            };

            if sender
                .send(Message::Text(json))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Handle incoming messages (client can send commands)
    let recv_task = tokio::spawn(async move {
        while let Some(result) = receiver.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    // Handle client commands
                    if let Err(e) = handle_command(&text) {
                        tracing::warn!("Failed to handle command: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Err(e) => {
                    tracing::warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
}

/// Handle a command from the WebSocket client
fn handle_command(command: &str) -> Result<(), CommandError> {
    // Parse as JSON to get the "cmd" field
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(command) {
        if let Some(cmd) = value.get("cmd").and_then(|v| v.as_str()) {
            match cmd {
                "ping" => {
                    // Ping is handled by the WebSocket protocol
                }
                "subscribe" => {
                    if let Some(events) = value.get("events").and_then(|v| v.as_array()) {
                        let event_names: Vec<String> = events
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(String::from)
                            .collect();
                        tracing::debug!("Client subscribed to events: {:?}", event_names);
                    }
                }
                "unsubscribe" => {
                    tracing::debug!("Client unsubscribed from events");
                }
                _ => {
                    tracing::warn!("Unknown command: {}", cmd);
                }
            }
            return Ok(());
        }
    }

    Err(CommandError::InvalidJson)
}

/// Command from the WebSocket client
#[derive(Deserialize)]
struct ClientCommand {
    /// Command type
    cmd: String,
    /// Event types (for subscribe)
    #[serde(default)]
    events: Vec<String>,
}

/// Command error
#[derive(Debug)]
enum CommandError {
    /// Invalid JSON
    InvalidJson,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJson => write!(f, "Invalid JSON"),
        }
    }
}

impl std::error::Error for CommandError {}

/// Server error
#[derive(Debug)]
pub enum ServerError {
    /// Failed to bind to address
    BindFailed(String),
    /// Server error
    ServeError(String),
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindFailed(msg) => write!(f, "Failed to bind: {}", msg),
            Self::ServeError(msg) => write!(f, "Server error: {}", msg),
        }
    }
}

impl std::error::Error for ServerError {}

/// Handle to a running server
pub struct ServerHandle {
    /// Shutdown sender
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
    /// Server task handle
    handle: Option<tokio::task::JoinHandle<Result<(), ServerError>>>,
}

impl ServerHandle {
    /// Stop the server
    pub async fn stop(mut self) -> Result<(), ServerError> {
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        if let Some(handle) = self.handle.take() {
            handle.await.map_err(|_| ServerError::ServeError("Join error".to_string()))??;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = InspectorConfig::default();
        assert_eq!(config.bind_addr, SocketAddr::from(([127, 0, 0, 1], 3000)));
        assert!(config.enable_cors);
    }

    #[test]
    fn test_config_builder() {
        let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
        let config = InspectorConfig::new()
            .with_bind_addr(addr)
            .with_cors(false)
            .with_event_buffer_capacity(200);

        assert_eq!(config.bind_addr, addr);
        assert!(!config.enable_cors);
        assert_eq!(config.event_buffer_capacity, 200);
    }

    #[test]
    fn test_server_create() {
        let server = InspectorServer::with_default_config();
        assert!(server.inspector().runtime_snapshot().state == crate::snapshot::RuntimeState::Stopped);
    }
}
