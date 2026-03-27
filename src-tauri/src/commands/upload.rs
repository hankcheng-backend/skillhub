use crate::commands::skills::DbState;
use crate::error::AppError;
use crate::services::token_store::KeyringTokenStore;
use crate::services::upload as upload_svc;
use tauri::State;

#[tauri::command]
pub async fn upload_skill(
    db: State<'_, DbState>,
    source_id: String,
    skill_id: String,
    force: Option<bool>,
) -> Result<(), AppError> {
    let token_store = KeyringTokenStore;
    upload_svc::upload_skill(
        db.inner(),
        &token_store,
        &source_id,
        &skill_id,
        force.unwrap_or(false),
    )
    .await
}
