use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database operation failed: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("Database migration failed: {0}")]
    Migration(String),

    #[error("File operation failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid skill.md frontmatter: {0} — ensure the file has valid YAML frontmatter with name and description fields")]
    Frontmatter(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Remote service error: {0}")]
    Remote(String),

    #[error("Authentication error: {0}")]
    OAuth(String),

    #[error("Access token expired for source: {0} — update the token to restore access")]
    TokenExpired(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let kind = match self {
            AppError::Db(_) => "Db",
            AppError::Migration(_) => "Migration",
            AppError::Io(_) => "Io",
            AppError::Frontmatter(_) => "Frontmatter",
            AppError::NotFound(_) => "NotFound",
            AppError::Conflict(_) => "Conflict",
            AppError::Remote(_) => "Remote",
            AppError::OAuth(_) => "OAuth",
            AppError::TokenExpired(_) => "TokenExpired",
            AppError::Internal(_) => "Internal",
        };
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("kind", kind)?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}
