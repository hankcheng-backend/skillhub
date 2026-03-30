use crate::commands::skills::DbState;
use crate::error::AppError;
use crate::services::install as install_svc;
use crate::services::token_store::KeyringTokenStore;
use tauri::State;

#[tauri::command]
pub async fn install_skill(
    db: State<'_, DbState>,
    source_id: String,
    folder_name: String,
    target_agent: String,
    force: Option<bool>,
) -> Result<(), AppError> {
    let token_store = KeyringTokenStore;
    install_svc::install_skill(
        db.inner(),
        &token_store,
        &source_id,
        &folder_name,
        &target_agent,
        force.unwrap_or(false),
    )
    .await
}
