use axum::{
    extract::State as AxumState,
    http::HeaderValue,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::{AllowOrigin, CorsLayer};

use super::tools;

type SharedDb = Arc<Mutex<Connection>>;

#[derive(Deserialize)]
struct McpRequest {
    method: String,
    params: serde_json::Value,
}

#[derive(Serialize)]
struct McpResponse {
    result: serde_json::Value,
}

#[derive(Serialize)]
struct McpError {
    error: String,
}

pub fn create_router(db: SharedDb) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([
            HeaderValue::from_static("tauri://localhost"),
            HeaderValue::from_static("http://localhost:1420"),
        ]))
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
        ]);

    Router::new()
        .route("/health", get(health))
        .route("/mcp", post(handle_mcp))
        .layer(cors)
        .with_state(db)
}

async fn health() -> impl IntoResponse {
    axum::http::StatusCode::OK
}

async fn handle_mcp(
    AxumState(db): AxumState<SharedDb>,
    Json(req): Json<McpRequest>,
) -> impl IntoResponse {
    let result = match req.method.as_str() {
        // Search & list
        "search_skills" => tools::search_skills(&db, req.params).await,
        "list_local_skills" => tools::list_local_skills(&db, req.params),
        // Skill content
        "get_skill_content" => tools::get_skill_content(&db, req.params).await,
        // Sync
        "sync_skill" => tools::sync_skill_tool(&db, req.params),
        "unsync_skill" => tools::unsync_skill_tool(&db, req.params),
        // Install / uninstall
        "install_skill" => tools::install_skill_tool(&db, req.params).await,
        "uninstall_skill" => tools::uninstall_skill_tool(&db, req.params),
        // Upload
        "upload_skill" => tools::upload_skill_tool(&db, req.params).await,
        // Sources
        "list_sources" => tools::list_sources_tool(&db),
        "add_source" => tools::add_source_tool(&db, req.params).await,
        "remove_source" => tools::remove_source_tool(&db, req.params),
        "browse_source" => tools::browse_source_tool(&db, req.params).await,
        // Agents
        "get_agents" => tools::get_agents_tool(&db),
        _ => Err(format!("Unknown method: {}", req.method)),
    };

    match result {
        Ok(val) => Json(McpResponse { result: val }).into_response(),
        Err(e) => Json(McpError { error: e }).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use rusqlite::Connection;
    use std::sync::{Arc, Mutex};
    use tower::util::ServiceExt;

    fn make_db() -> SharedDb {
        Arc::new(Mutex::new(
            Connection::open_in_memory().expect("open in-memory DB"),
        ))
    }

    #[tokio::test]
    async fn health_endpoint_allows_tauri_localhost_origin() {
        let app = create_router(make_db());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .method("GET")
                    .header("Origin", "tauri://localhost")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("request health endpoint");

        assert!(response.status().is_success());
        let allow_origin = response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok());
        assert_eq!(allow_origin, Some("tauri://localhost"));
    }

    #[tokio::test]
    async fn health_endpoint_allows_dev_origin() {
        let app = create_router(make_db());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .method("GET")
                    .header("Origin", "http://localhost:1420")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("request health endpoint");

        assert!(response.status().is_success());
        let allow_origin = response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok());
        assert_eq!(allow_origin, Some("http://localhost:1420"));
    }

    #[tokio::test]
    async fn health_endpoint_rejects_unknown_origin() {
        let app = create_router(make_db());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .method("GET")
                    .header("Origin", "https://evil.example.com")
                    .body(Body::empty())
                    .expect("build request"),
            )
            .await
            .expect("request health endpoint");

        // Status is still 200 but CORS header must not reflect the disallowed origin
        assert!(response.status().is_success());
        let allow_origin = response
            .headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok());
        assert_ne!(allow_origin, Some("https://evil.example.com"));
    }
}
