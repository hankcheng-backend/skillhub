pub mod gdrive;
pub mod gitlab;
pub mod oauth;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteSkill {
    pub folder_name: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub source_id: String,
    pub source_name: String,
    pub updated_at: Option<String>,
    pub updated_by: Option<String>,
}
