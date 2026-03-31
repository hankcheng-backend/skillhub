use crate::commands::skills::DbState;
use crate::db::models::Source;
use crate::error::AppError;
use crate::remote::RemoteSkill;
use crate::services::sources as sources_svc;
use crate::services::token_store::KeyringTokenStore;
use tauri::State;

#[tauri::command]
pub fn list_sources(db: State<'_, DbState>) -> Result<Vec<Source>, AppError> {
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    Source::all(&conn).map_err(AppError::from)
}

#[tauri::command]
pub async fn add_source(
    db: State<'_, DbState>,
    name: String,
    source_type: String,
    url: Option<String>,
    folder_id: Option<String>,
    token: Option<String>,
) -> Result<Source, AppError> {
    let token_store = KeyringTokenStore;
    sources_svc::add_source(
        db.inner(),
        &token_store,
        &name,
        &source_type,
        url.as_deref(),
        folder_id.as_deref(),
        token.as_deref(),
    )
    .await
}

#[tauri::command]
pub fn remove_source(db: State<'_, DbState>, source_id: String) -> Result<(), AppError> {
    let token_store = KeyringTokenStore;
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    sources_svc::remove_source(&conn, &token_store, &source_id)
}

#[tauri::command]
pub async fn update_source_token(
    db: State<'_, DbState>,
    source_id: String,
    new_token: String,
) -> Result<(), AppError> {
    let token_store = KeyringTokenStore;
    let conn = db
        .lock()
        .map_err(|e| AppError::Internal(format!("DB lock poisoned: {}", e)))?;
    sources_svc::update_source_token(&conn, &token_store, &source_id, &new_token)
}

#[tauri::command]
pub async fn browse_source(
    db: State<'_, DbState>,
    source_id: String,
) -> Result<Vec<RemoteSkill>, AppError> {
    let token_store = KeyringTokenStore;
    sources_svc::browse_source(db.inner(), &token_store, &source_id).await
}

#[tauri::command]
pub async fn get_remote_skill_content(
    db: State<'_, DbState>,
    source_id: String,
    folder_name: String,
) -> Result<Option<String>, AppError> {
    let token_store = KeyringTokenStore;
    sources_svc::get_remote_skill_content(db.inner(), &token_store, &source_id, &folder_name).await
}
