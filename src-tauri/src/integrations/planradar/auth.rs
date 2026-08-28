use super::client::PlanradarApiClient;
use super::projects::{list_projects_core, PlanradarListProjectsInput};
use super::shared::{
    delete_api_token, load_store_or_error, normalize_base_url, peek_api_token, save_store_or_error,
    store_api_token, PlanradarApiError, PlanradarApiErrorCode, PreviousToken,
};
use crate::integrations::local_store::LocalStore;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarConnectRequest {
    pub base_url: String,
    pub customer_id: String,
    pub api_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarConnectionStatus {
    pub has_token: bool,
    pub customer_id: String,
}

/// Stores the API token in the OS keychain and the non-secret base URL plus Customer ID in the
/// local config store.
#[tauri::command]
#[specta::specta]
pub async fn planradar_connect(
    app: tauri::AppHandle,
    request: PlanradarConnectRequest,
) -> Result<PlanradarConnectionStatus, PlanradarApiError> {
    let credentials = validate_connect_request(&request)?;

    // Probe before persisting: a single-record list authenticates the token and exercises the
    // Customer ID path segment.
    let client = PlanradarApiClient::new(&credentials.base_url)?;
    list_projects_core(
        &client,
        &credentials.api_token,
        &credentials.customer_id,
        &PlanradarListProjectsInput {
            pagesize: Some(1),
            ..PlanradarListProjectsInput::default()
        },
    )
    .await
    .map_err(remap_probe_error)?;

    persist_credentials(&TauriConnectionStore { app }, &credentials)?;

    Ok(PlanradarConnectionStatus {
        // Not re-read from the keychain: a transient read error must not fail a persisted connect.
        has_token: true,
        customer_id: credentials.customer_id,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PlanradarCredentials {
    pub base_url: String,
    pub customer_id: String,
    pub api_token: String,
}

fn validate_connect_request(
    request: &PlanradarConnectRequest,
) -> Result<PlanradarCredentials, PlanradarApiError> {
    let base_url = normalize_base_url(&request.base_url)?;

    let customer_id = request.customer_id.trim().to_string();
    if customer_id.is_empty() {
        return Err(PlanradarApiError::new(
            PlanradarApiErrorCode::MissingCustomerId,
            None,
            "Die Planradar Customer ID darf nicht leer sein.",
            "planradar_connect mit leerer customer_id aufgerufen",
        ));
    }

    let api_token = request.api_token.trim().to_string();
    if api_token.is_empty() {
        return Err(PlanradarApiError::new(
            PlanradarApiErrorCode::MissingToken,
            None,
            "Das Planradar-Token darf nicht leer sein.",
            "planradar_connect mit leerem api_token aufgerufen",
        ));
    }

    Ok(PlanradarCredentials {
        base_url,
        customer_id,
        api_token,
    })
}

/// The keychain and the config store are separate persistence layers, so [`persist_credentials`]
/// is written against this trait to keep its write ordering and rollback under test.
pub(super) trait ConnectionStore {
    fn load(&self) -> Result<LocalStore, PlanradarApiError>;
    fn save(&self, store: LocalStore) -> Result<(), PlanradarApiError>;
    fn peek_token(&self) -> PreviousToken;
    fn store_token(&self, token: &str) -> Result<(), PlanradarApiError>;
    fn delete_token(&self) -> Result<(), PlanradarApiError>;
}

struct TauriConnectionStore {
    app: tauri::AppHandle,
}

impl ConnectionStore for TauriConnectionStore {
    fn load(&self) -> Result<LocalStore, PlanradarApiError> {
        load_store_or_error(self.app.clone())
    }

    fn save(&self, store: LocalStore) -> Result<(), PlanradarApiError> {
        save_store_or_error(self.app.clone(), store)
    }

    fn peek_token(&self) -> PreviousToken {
        peek_api_token()
    }

    fn store_token(&self, token: &str) -> Result<(), PlanradarApiError> {
        store_api_token(token)
    }

    fn delete_token(&self) -> Result<(), PlanradarApiError> {
        delete_api_token()
    }
}

pub(super) fn persist_credentials(
    backend: &dyn ConnectionStore,
    credentials: &PlanradarCredentials,
) -> Result<(), PlanradarApiError> {
    // Load the store before touching the keychain so a store-read failure cannot orphan a token.
    let mut store = backend.load()?;
    let previous_token = backend.peek_token();

    backend.store_token(&credentials.api_token)?;

    store.api_endpoints.planradar_base_url = credentials.base_url.clone();
    store.api_endpoints.planradar_customer_id = credentials.customer_id.clone();
    if let Err(error) = backend.save(store) {
        // On an unknown previous token, leave the keychain alone rather than risk deleting a
        // token that is still valid.
        let _ = match &previous_token {
            PreviousToken::Present(previous) => backend.store_token(previous),
            PreviousToken::Absent => backend.delete_token(),
            PreviousToken::Unknown => Ok(()),
        };
        return Err(error);
    }

    Ok(())
}

/// A raw "project not found" or "token invalid" is confusing in a credentials dialog; a wrong
/// Customer ID also surfaces as a 404 there.
fn remap_probe_error(error: PlanradarApiError) -> PlanradarApiError {
    match error.code {
        PlanradarApiErrorCode::Unauthorized | PlanradarApiErrorCode::NotFound => {
            PlanradarApiError::new(
                error.code,
                error.http_status,
                "Verbindung fehlgeschlagen. Bitte Customer ID und API-Token prüfen.",
                error.technical_message,
            )
        }
        _ => error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[derive(Debug, PartialEq, Eq)]
    enum Call {
        Load,
        Peek,
        StoreToken(String),
        DeleteToken,
        Save(String, String),
    }

    #[derive(Default)]
    struct FakeStore {
        calls: RefCell<Vec<Call>>,
        previous_token: Option<PreviousTokenKind>,
        load_fails: bool,
        save_fails: bool,
        store_token_fails: bool,
    }

    #[derive(Clone, Copy)]
    enum PreviousTokenKind {
        Present,
        Absent,
        Unknown,
    }

    impl FakeStore {
        fn with_previous(previous_token: PreviousTokenKind) -> Self {
            Self {
                previous_token: Some(previous_token),
                ..Self::default()
            }
        }

        fn calls(&self) -> std::cell::Ref<'_, Vec<Call>> {
            self.calls.borrow()
        }
    }

    impl ConnectionStore for FakeStore {
        fn load(&self) -> Result<LocalStore, PlanradarApiError> {
            self.calls.borrow_mut().push(Call::Load);
            if self.load_fails {
                return Err(error("load failed"));
            }
            Ok(LocalStore::default())
        }

        fn save(&self, store: LocalStore) -> Result<(), PlanradarApiError> {
            self.calls.borrow_mut().push(Call::Save(
                store.api_endpoints.planradar_base_url,
                store.api_endpoints.planradar_customer_id,
            ));
            if self.save_fails {
                return Err(error("save failed"));
            }
            Ok(())
        }

        fn peek_token(&self) -> PreviousToken {
            self.calls.borrow_mut().push(Call::Peek);
            match self.previous_token {
                Some(PreviousTokenKind::Present) => PreviousToken::Present("alt".to_string()),
                Some(PreviousTokenKind::Unknown) => PreviousToken::Unknown,
                _ => PreviousToken::Absent,
            }
        }

        fn store_token(&self, token: &str) -> Result<(), PlanradarApiError> {
            self.calls
                .borrow_mut()
                .push(Call::StoreToken(token.to_string()));
            if self.store_token_fails {
                return Err(error("keychain write failed"));
            }
            Ok(())
        }

        fn delete_token(&self) -> Result<(), PlanradarApiError> {
            self.calls.borrow_mut().push(Call::DeleteToken);
            Ok(())
        }
    }

    fn error(technical_message: &str) -> PlanradarApiError {
        PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidConfiguration,
            None,
            "Fehler",
            technical_message,
        )
    }

    fn credentials() -> PlanradarCredentials {
        PlanradarCredentials {
            base_url: "https://www.planradar.com".to_string(),
            customer_id: "1234".to_string(),
            api_token: "neu".to_string(),
        }
    }

    fn connect_request(customer_id: &str, api_token: &str) -> PlanradarConnectRequest {
        PlanradarConnectRequest {
            base_url: "https://www.planradar.com".to_string(),
            customer_id: customer_id.to_string(),
            api_token: api_token.to_string(),
        }
    }

    #[test]
    fn validate_trims_and_normalizes_the_request() {
        let validated = validate_connect_request(&PlanradarConnectRequest {
            base_url: "https://www.planradar.com/api/".to_string(),
            customer_id: "  1234 ".to_string(),
            api_token: " geheim ".to_string(),
        })
        .expect("request should validate");

        assert_eq!(validated, credentials_with_token("geheim"));
    }

    fn credentials_with_token(api_token: &str) -> PlanradarCredentials {
        PlanradarCredentials {
            api_token: api_token.to_string(),
            ..credentials()
        }
    }

    #[test]
    fn validate_rejects_blank_customer_id() {
        let error = validate_connect_request(&connect_request("   ", "geheim"))
            .expect_err("blank customer id should fail");
        assert_eq!(error.code, PlanradarApiErrorCode::MissingCustomerId);
    }

    #[test]
    fn validate_rejects_blank_api_token() {
        let error = validate_connect_request(&connect_request("1234", "  "))
            .expect_err("blank token should fail");
        assert_eq!(error.code, PlanradarApiErrorCode::MissingToken);
    }

    #[test]
    fn persist_loads_the_store_before_writing_the_token() {
        let backend = FakeStore::with_previous(PreviousTokenKind::Absent);

        persist_credentials(&backend, &credentials()).expect("persist should succeed");

        assert_eq!(
            *backend.calls(),
            vec![
                Call::Load,
                Call::Peek,
                Call::StoreToken("neu".to_string()),
                Call::Save("https://www.planradar.com".to_string(), "1234".to_string()),
            ]
        );
    }

    #[test]
    fn persist_writes_no_token_when_the_store_cannot_be_loaded() {
        let backend = FakeStore {
            load_fails: true,
            ..FakeStore::default()
        };

        persist_credentials(&backend, &credentials()).expect_err("load failure should propagate");

        assert_eq!(*backend.calls(), vec![Call::Load]);
    }

    #[test]
    fn persist_does_not_save_the_store_when_the_token_cannot_be_written() {
        let backend = FakeStore {
            store_token_fails: true,
            ..FakeStore::with_previous(PreviousTokenKind::Absent)
        };

        persist_credentials(&backend, &credentials())
            .expect_err("keychain failure should propagate");

        assert!(!backend
            .calls()
            .iter()
            .any(|call| matches!(call, Call::Save(_, _))));
    }

    #[test]
    fn persist_restores_the_previous_token_when_the_store_write_fails() {
        let backend = FakeStore {
            save_fails: true,
            ..FakeStore::with_previous(PreviousTokenKind::Present)
        };

        persist_credentials(&backend, &credentials()).expect_err("save failure should propagate");

        assert_eq!(
            backend.calls().last(),
            Some(&Call::StoreToken("alt".to_string()))
        );
    }

    #[test]
    fn persist_deletes_the_new_token_when_none_was_stored_before() {
        let backend = FakeStore {
            save_fails: true,
            ..FakeStore::with_previous(PreviousTokenKind::Absent)
        };

        persist_credentials(&backend, &credentials()).expect_err("save failure should propagate");

        assert_eq!(backend.calls().last(), Some(&Call::DeleteToken));
    }

    #[test]
    fn persist_leaves_an_unreadable_keychain_untouched_when_the_store_write_fails() {
        let backend = FakeStore {
            save_fails: true,
            ..FakeStore::with_previous(PreviousTokenKind::Unknown)
        };

        persist_credentials(&backend, &credentials()).expect_err("save failure should propagate");

        // Deleting here would wipe a token that may still be valid.
        assert!(!backend.calls().contains(&Call::DeleteToken));
        assert_eq!(
            backend
                .calls()
                .iter()
                .filter(|call| matches!(call, Call::StoreToken(_)))
                .count(),
            1
        );
    }

    #[test]
    fn probe_failures_point_at_the_credentials() {
        for code in [
            PlanradarApiErrorCode::Unauthorized,
            PlanradarApiErrorCode::NotFound,
        ] {
            let remapped =
                remap_probe_error(PlanradarApiError::new(code, Some(404), "egal", "technisch"));
            assert!(remapped.user_message.contains("Customer ID"));
            assert_eq!(remapped.technical_message, "technisch");
        }
    }

    #[test]
    fn probe_failures_that_are_not_credential_related_keep_their_message() {
        let remapped = remap_probe_error(PlanradarApiError::new(
            PlanradarApiErrorCode::ServerError,
            Some(503),
            "Planradar ist aktuell nicht erreichbar.",
            "technisch",
        ));
        assert_eq!(
            remapped.user_message,
            "Planradar ist aktuell nicht erreichbar."
        );
    }
}
