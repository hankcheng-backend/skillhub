use crate::error::AppError;

/// Abstraction over keychain/token storage for testability.
/// Production code uses `KeyringTokenStore`; tests can use `InMemoryTokenStore`.
pub trait TokenStore: Send + Sync {
    fn get_token(&self, service: &str, key: &str) -> Result<String, AppError>;
    fn store_token(&self, service: &str, key: &str, token: &str) -> Result<(), AppError>;
    /// Delete a stored token. Best-effort — errors are silently ignored by callers.
    fn delete_token(&self, service: &str, key: &str) -> Result<(), AppError>;
}

/// Production implementation backed by the OS keychain via the `keyring` crate.
pub struct KeyringTokenStore;

impl TokenStore for KeyringTokenStore {
    fn get_token(&self, service: &str, key: &str) -> Result<String, AppError> {
        let entry =
            keyring::Entry::new(service, key).map_err(|e| AppError::OAuth(e.to_string()))?;
        entry
            .get_password()
            .map_err(|e| AppError::OAuth(e.to_string()))
    }

    fn store_token(&self, service: &str, key: &str, token: &str) -> Result<(), AppError> {
        let entry =
            keyring::Entry::new(service, key).map_err(|e| AppError::OAuth(e.to_string()))?;
        entry
            .set_password(token)
            .map_err(|e| AppError::OAuth(e.to_string()))
    }

    fn delete_token(&self, service: &str, key: &str) -> Result<(), AppError> {
        if let Ok(entry) = keyring::Entry::new(service, key) {
            let _ = entry.delete_credential();
        }
        Ok(())
    }
}

/// In-memory implementation for use in tests — no OS keychain required.
#[derive(Default)]
pub struct InMemoryTokenStore {
    tokens: std::sync::Mutex<std::collections::HashMap<String, String>>,
}

impl TokenStore for InMemoryTokenStore {
    fn get_token(&self, service: &str, key: &str) -> Result<String, AppError> {
        let k = format!("{}:{}", service, key);
        self.tokens
            .lock()
            .unwrap()
            .get(&k)
            .cloned()
            .ok_or_else(|| AppError::OAuth(format!("No token for {}", k)))
    }

    fn store_token(&self, service: &str, key: &str, token: &str) -> Result<(), AppError> {
        let k = format!("{}:{}", service, key);
        self.tokens.lock().unwrap().insert(k, token.to_string());
        Ok(())
    }

    fn delete_token(&self, service: &str, key: &str) -> Result<(), AppError> {
        let k = format!("{}:{}", service, key);
        self.tokens.lock().unwrap().remove(&k);
        Ok(())
    }
}
