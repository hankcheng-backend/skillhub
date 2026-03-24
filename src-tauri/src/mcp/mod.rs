pub mod router;
pub mod tools;

use crate::error::AppError;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub async fn start_server(db: Arc<Mutex<Connection>>, port: u16) -> Result<(), AppError> {
    let app = router::create_router(db);
    let addr = format!("127.0.0.1:{}", port);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| AppError::Io(e))?;

    log::info!("MCP Server listening on {}", addr);

    axum::serve(listener, app).await.map_err(|e| {
        AppError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    Ok(())
}
