// Caret Inspector - HTTP API handlers
//
// Copyright (c) 2025 Caret Contributors
//
// Licensed under the MIT License:
// https://opensource.org/licenses/MIT

use crate::{snapshot::*, InspectorHandle, EventStream};
use axum::{
    extract::{State, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

/// API error type
#[derive(Debug)]
pub enum ApiError {
    /// Inspector not available
    InspectorNotAvailable,
    /// Invalid request
    InvalidRequest(String),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InspectorNotAvailable => write!(f, "Inspector not available"),
            Self::InvalidRequest(msg) => write!(f, "{}", msg),
            Self::Internal(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::InspectorNotAvailable => {
                (StatusCode::SERVICE_UNAVAILABLE, "Inspector not available".to_string())
            }
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(serde_json::json!({
            "error": message,
        }));

        (status, body).into_response()
    }
}

/// Inspector API state
#[derive(Clone)]
pub struct ApiState {
    /// Inspector handle
    pub inspector: InspectorHandle,
    /// Event stream
    pub events: EventStream,
}

/// Inspector API
///
/// Provides REST endpoints for accessing runtime state.
pub struct InspectorApi {
    /// Router for the API
    router: Router,
}

impl InspectorApi {
    /// Create a new inspector API
    pub fn new(state: ApiState) -> Self {
        let router = Self::routes(state);
        Self { router }
    }

    /// Get the API router
    pub fn router(&self) -> Router {
        self.router.clone()
    }

    /// Create API routes
    fn routes(state: ApiState) -> Router {
        Router::new()
            .route("/api", get(get_api_info))
            .route("/api/runtime", get(get_runtime))
            .route("/api/graph", get(get_graph))
            .route("/api/nodes", get(get_nodes))
            .route("/api/nodes/:id", get(get_node))
            .route("/api/metrics", get(get_metrics))
            .route("/api/stats", get(get_stats))
            .with_state(state)
    }
}

/// GET /api - API information
async fn get_api_info() -> impl IntoResponse {
    Json(serde_json::json!({
        "name": "Caret Inspector API",
        "version": "0.1.0",
        "endpoints": [
            "GET /api/runtime - Get runtime state",
            "GET /api/graph - Get graph topology",
            "GET /api/nodes - Get all nodes",
            "GET /api/nodes/:id - Get specific node",
            "GET /api/metrics - Get metrics",
            "GET /api/stats - Get statistics",
            "WS /api/stream - Event stream (WebSocket)",
        ]
    }))
}

/// GET /api/runtime - Get runtime state
async fn get_runtime(State(state): State<ApiState>) -> Result<Json<RuntimeSnapshot>, ApiError> {
    let snapshot = state.inspector.snapshot();
    Ok(Json(snapshot.runtime))
}

/// GET /api/graph - Get graph topology
async fn get_graph(State(state): State<ApiState>) -> Result<Json<GraphSnapshot>, ApiError> {
    let snapshot = state.inspector.snapshot();
    Ok(Json(snapshot.graph))
}

/// GET /api/nodes - Get all nodes
async fn get_nodes(State(state): State<ApiState>) -> Result<Json<Vec<NodeSnapshot>>, ApiError> {
    let snapshot = state.inspector.snapshot();
    Ok(Json(snapshot.graph.nodes))
}

/// GET /api/nodes/:id - Get specific node
async fn get_node(
    State(state): State<ApiState>,
    Path(id): Path<u64>,
) -> Result<Json<NodeSnapshot>, ApiError> {
    let snapshot = state.inspector.snapshot();
    snapshot
        .graph
        .nodes
        .into_iter()
        .find(|n| n.id == id)
        .map(Json)
        .ok_or_else(|| ApiError::InvalidRequest(format!("Node {} not found", id)))
}

/// GET /api/metrics - Get all metrics
async fn get_metrics(
    State(state): State<ApiState>,
    Query(query): Query<MetricsQuery>,
) -> Result<Json<Vec<MetricSnapshot>>, ApiError> {
    let mut metrics = state.inspector.snapshot().metrics;

    // Filter by type if specified
    if let Some(metric_type) = query.r#type {
        let filter_type = match metric_type.as_str() {
            "counter" => Some(MetricType::Counter),
            "gauge" => Some(MetricType::Gauge),
            "histogram" => Some(MetricType::Histogram),
            _ => return Err(ApiError::InvalidRequest(format!("Invalid metric type: {}", metric_type))),
        };
        if let Some(filter) = filter_type {
            metrics.retain(|m| m.metric_type == filter);
        }
    }

    // Filter by name pattern if specified
    if let Some(pattern) = query.name {
        metrics.retain(|m| m.name.contains(&pattern));
    }

    Ok(Json(metrics))
}

/// Query parameters for metrics endpoint
#[derive(Deserialize)]
struct MetricsQuery {
    /// Filter by metric type
    r#type: Option<String>,
    /// Filter by name pattern
    name: Option<String>,
}

/// GET /api/stats - Get aggregated statistics
async fn get_stats(State(state): State<ApiState>) -> Result<Json<StatsResponse>, ApiError> {
    let snapshot = state.inspector.snapshot();

    let total_packets: u64 = snapshot.graph.nodes.iter().map(|n| n.packets_processed).sum();
    let total_dropped: u64 = snapshot.graph.nodes.iter().map(|n| n.packets_dropped).sum();

    let queue_depths: Vec<usize> = snapshot
        .graph
        .nodes
        .iter()
        .flat_map(|n| n.inputs.iter().map(|p| p.depth))
        .collect();

    let avg_queue_depth = if !queue_depths.is_empty() {
        queue_depths.iter().sum::<usize>() / queue_depths.len()
    } else {
        0
    };

    let max_queue_depth = queue_depths.into_iter().max().unwrap_or(0);

    let response = StatsResponse {
        uptime_ms: snapshot.runtime.tick * 10, // Approximate
        total_packets,
        total_dropped,
        nodes_total: snapshot.graph.nodes.len() as u32,
        nodes_running: snapshot.runtime.running_nodes as u32,
        nodes_completed: snapshot.runtime.completed_nodes as u32,
        nodes_errored: snapshot.runtime.errored_nodes as u32,
        avg_queue_depth,
        max_queue_depth,
        current_tick: snapshot.runtime.tick,
    };

    Ok(Json(response))
}

/// Statistics response
#[derive(Serialize)]
struct StatsResponse {
    /// Uptime in milliseconds
    uptime_ms: u64,
    /// Total packets processed
    total_packets: u64,
    /// Total packets dropped
    total_dropped: u64,
    /// Total nodes
    nodes_total: u32,
    /// Currently running nodes
    nodes_running: u32,
    /// Completed nodes
    nodes_completed: u32,
    /// Errored nodes
    nodes_errored: u32,
    /// Average queue depth
    avg_queue_depth: usize,
    /// Maximum queue depth
    max_queue_depth: usize,
    /// Current tick
    current_tick: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Inspector;

    #[test]
    fn test_api_error_display() {
        let err = ApiError::InvalidRequest("test error".to_string());
        assert_eq!(err.to_string(), "test error");
    }

    #[test]
    fn test_metrics_query_deserialize() {
        let query: MetricsQuery = serde_json::from_str("{}").unwrap();
        assert!(query.r#type.is_none());
        assert!(query.name.is_none());

        let query: MetricsQuery =
            serde_json::from_str(r#"{"type":"counter","name":"test"}"#).unwrap();
        assert_eq!(query.r#type.unwrap(), "counter");
        assert_eq!(query.name.unwrap(), "test");
    }

    #[test]
    fn test_stats_response_serialization() {
        let response = StatsResponse {
            uptime_ms: 1000,
            total_packets: 100,
            total_dropped: 5,
            nodes_total: 10,
            nodes_running: 5,
            nodes_completed: 3,
            nodes_errored: 1,
            avg_queue_depth: 10,
            max_queue_depth: 50,
            current_tick: 100,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"total_packets\":100"));
        assert!(json.contains("\"nodes_total\":10"));
    }
}
