use axum::{
    extract::State as AxumState,
    http::header::{ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
    Router::new()
        .route("/health", get(health))
        .route("/mcp", post(handle_mcp))
        .with_state(db)
}

async fn health() -> impl IntoResponse {
    (
        [
            (ACCESS_CONTROL_ALLOW_ORIGIN, "*"),
            (ACCESS_CONTROL_ALLOW_METHODS, "GET"),
        ],
        axum::http::StatusCode::OK,
    )
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
        "add_source" => tools::add_source_tool(&db, req.params),
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

    #[tokio::test]
    async fn health_endpoint_exposes_cors_header_for_ui_polling() {
        let db = Arc::new(Mutex::new(
            Connection::open_in_memory().expect("open in-memory DB"),
        ));
        let app = create_router(db);

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
            .and_then(|value| value.to_str().ok());
        assert_eq!(allow_origin, Some("*"));
    }
}
