use super::RemoteSkill;
use crate::error::AppError;

pub async fn list_skills(_folder_id: &str, _token: &str) -> Result<Vec<RemoteSkill>, AppError> {
    Err(AppError::Remote(
        "Google Drive integration not yet implemented".into(),
    ))
}

pub async fn download_skill(
    _folder_id: &str,
    _skill_folder_name: &str,
    _token: &str,
    _dest: &std::path::Path,
) -> Result<(), AppError> {
    Err(AppError::Remote(
        "Google Drive download not yet implemented".into(),
    ))
}
