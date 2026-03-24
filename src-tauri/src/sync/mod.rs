use crate::error::AppError;
use std::path::Path;

/// Create a symlink (or junction on Windows) from `link` pointing to `target`.
pub fn create_sync_link(target: &Path, link: &Path) -> Result<(), AppError> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }

    if link.exists() || link.symlink_metadata().is_ok() {
        return Err(AppError::Conflict(format!(
            "Path already exists: {}",
            link.display()
        )));
    }

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target, link)?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(target, link).map_err(|e| AppError::Io(e))?;
    }

    Ok(())
}

/// Remove a symlink (or junction on Windows).
pub fn remove_sync_link(link: &Path) -> Result<(), AppError> {
    let metadata = std::fs::symlink_metadata(link)?;
    if !metadata.is_symlink() {
        return Err(AppError::Conflict(format!(
            "Not a symlink: {}",
            link.display()
        )));
    }

    #[cfg(unix)]
    {
        std::fs::remove_file(link)?;
    }

    #[cfg(windows)]
    {
        std::fs::remove_dir(link)?;
    }

    Ok(())
}
