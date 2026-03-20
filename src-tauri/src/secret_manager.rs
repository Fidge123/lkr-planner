#[derive(Debug, PartialEq, Serialize, Deserialize, Type)]
#[serde(tag = "type", content = "message")]
pub enum SecretError {
    AccessDenied(String),
    NotFound,
    Other(String),
}

impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecretError::AccessDenied(msg) => write!(f, "Keychain access denied: {}", msg),
            SecretError::NotFound => write!(f, "Token not found"),
            SecretError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for SecretError {}

use keyring::Entry;

use keyring::Error as KeyringError;
use serde::{Deserialize, Serialize};
use specta::Type;

fn map_keyring_error(err: KeyringError) -> SecretError {
    match err {
        KeyringError::NoEntry => SecretError::NotFound,
        KeyringError::PlatformFailure(ref e) => {
            let error_msg = e.to_string().to_lowercase();
            // A simple heuristic for mapping common OS access denied messages
            if error_msg.contains("access denied") 
                || error_msg.contains("auth") 
                || error_msg.contains("not permitted")
                || error_msg.contains("user canceled")
            {
                SecretError::AccessDenied(err.to_string())
            } else {
                SecretError::Other(err.to_string())
            }
        }
        _ => SecretError::Other(err.to_string()),
    }
}

#[tauri::command]
#[specta::specta]
pub fn set_token(service: &str, username: &str, token: &str) -> Result<(), SecretError> {
    let entry = Entry::new(service, username).map_err(|e| map_keyring_error(e))?;
    entry.set_password(token).map_err(map_keyring_error)
}

#[tauri::command]
#[specta::specta]
pub fn get_token(service: &str, username: &str) -> Result<String, SecretError> {
    let entry = Entry::new(service, username).map_err(|e| map_keyring_error(e))?;
    entry.get_password().map_err(map_keyring_error)
}

#[tauri::command]
#[specta::specta]
pub fn delete_token(service: &str, username: &str) -> Result<(), SecretError> {
    let entry = Entry::new(service, username).map_err(|e| map_keyring_error(e))?;
    entry.delete_credential().map_err(map_keyring_error)
}

#[tauri::command]
#[specta::specta]
pub fn check_token(service: &str, username: &str) -> Result<bool, SecretError> {
    let entry = Entry::new(service, username).map_err(|e| map_keyring_error(e))?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(KeyringError::NoEntry) => Ok(false),
        Err(e) => Err(map_keyring_error(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use keyring::mock;

    fn with_mock_keyring<T>(f: impl FnOnce() -> T) -> T {
        keyring::set_default_credential_builder(mock::default_credential_builder());
        f()
    }

    #[test]
    fn get_nonexistent_token_returns_not_found() {
        with_mock_keyring(|| {
            let err = get_token("nonexistent_service", "nobody").unwrap_err();
            assert_eq!(err, SecretError::NotFound);
        });
    }

    #[test]
    fn check_token_returns_false_when_not_set() {
        with_mock_keyring(|| {
            let result = check_token("absent_service", "nobody").unwrap();
            assert!(!result);
        });
    }

    #[test]
    fn map_keyring_error_maps_no_entry_to_not_found() {
        let err = map_keyring_error(keyring::Error::NoEntry);
        assert_eq!(err, SecretError::NotFound);
    }

    /// Verify that set + get on the same Entry object works with the mock backend.
    /// Note: the mock uses EntryOnly persistence, so two separate Entry::new calls
    /// for the same service/user will NOT share state.
    #[test]
    fn set_and_get_on_same_entry_with_mock() {
        with_mock_keyring(|| {
            let entry = Entry::new("lkr-test", "tester").unwrap();
            entry.set_password("my-secret").unwrap();
            let pw = entry.get_password().unwrap();
            assert_eq!(pw, "my-secret");
        });
    }
}
