use crate::error::AppError;

pub async fn start_oauth_flow(
    _auth_url: &str,
    _token_url: &str,
    _client_id: &str,
    _scopes: &[&str],
) -> Result<(String, String), AppError> {
    Err(AppError::OAuth("OAuth not yet implemented".into()))
}

pub fn store_token(service: &str, key: &str, token: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(service, key).map_err(|e| AppError::OAuth(e.to_string()))?;
    entry
        .set_password(token)
        .map_err(|e| AppError::OAuth(e.to_string()))?;
    Ok(())
}

pub fn get_token(service: &str, key: &str) -> Result<String, AppError> {
    let entry = keyring::Entry::new(service, key).map_err(|e| AppError::OAuth(e.to_string()))?;
    entry
        .get_password()
        .map_err(|e| AppError::OAuth(e.to_string()))
}
