use super::types::{ZepError, ZepErrorCode, ZepStoredCredentials};
use crate::secret_manager;
use std::sync::{OnceLock, RwLock};

const ZEP_KEYCHAIN_SERVICE: &str = "lkr-planner-zep";
const ZEP_KEYCHAIN_ACCOUNT: &str = "LKR Planner ZEP Admin";

pub(super) fn save_zep_credentials_to_keychain(
    username: &str,
    password: &str,
) -> Result<(), ZepError> {
    let payload = serde_json::to_string(&ZepStoredCredentials {
        username: username.to_string(),
        password: password.to_string(),
    })
    .map_err(|e| {
        ZepError::new(
            ZepErrorCode::KeychainError,
            "Die ZEP-Zugangsdaten konnten nicht gespeichert werden.",
            format!("Serialisierung fehlgeschlagen: {e}"),
        )
    })?;

    secret_manager::set_token(ZEP_KEYCHAIN_SERVICE, ZEP_KEYCHAIN_ACCOUNT, &payload).map_err(
        |e| {
            ZepError::new(
                ZepErrorCode::KeychainError,
                "Die ZEP-Zugangsdaten konnten nicht im Keychain gespeichert werden (Zugriff verweigert?).",
                e.to_string(),
            )
        },
    )?;

    credential_cache().store(ZepStoredCredentials {
        username: username.to_string(),
        password: password.to_string(),
    });
    Ok(())
}

pub(crate) fn load_zep_credentials() -> Result<ZepStoredCredentials, ZepError> {
    credential_cache().get_or_load(load_zep_credentials_from_keychain)
}

#[derive(Default)]
struct CredentialCache {
    cached: RwLock<Option<ZepStoredCredentials>>,
}

impl CredentialCache {
    fn get_or_load(
        &self,
        load: impl FnOnce() -> Result<ZepStoredCredentials, ZepError>,
    ) -> Result<ZepStoredCredentials, ZepError> {
        if let Some(cached) = self
            .cached
            .read()
            .expect("credential cache poisoned")
            .clone()
        {
            return Ok(cached);
        }

        let loaded = load()?;
        self.store(loaded.clone());
        Ok(loaded)
    }

    fn store(&self, credentials: ZepStoredCredentials) {
        *self.cached.write().expect("credential cache poisoned") = Some(credentials);
    }
}

fn credential_cache() -> &'static CredentialCache {
    static CACHE: OnceLock<CredentialCache> = OnceLock::new();
    CACHE.get_or_init(CredentialCache::default)
}

fn load_zep_credentials_from_keychain() -> Result<ZepStoredCredentials, ZepError> {
    let json_str = secret_manager::get_token(ZEP_KEYCHAIN_SERVICE, ZEP_KEYCHAIN_ACCOUNT).map_err(
        |e| match e {
            secret_manager::SecretError::NotFound => ZepError::new(
                ZepErrorCode::MissingCredentials,
                "Keine ZEP-Zugangsdaten hinterlegt. Bitte ZEP-Verbindung konfigurieren.",
                "Kein Keychain-Eintrag für ZEP-Zugangsdaten.",
            ),
            _ => ZepError::new(
                ZepErrorCode::KeychainError,
                "Auf die ZEP-Zugangsdaten im Keychain konnte nicht zugegriffen werden.",
                e.to_string(),
            ),
        },
    )?;

    serde_json::from_str::<ZepStoredCredentials>(&json_str).map_err(|e| {
        ZepError::new(
            ZepErrorCode::KeychainError,
            "Die gespeicherten ZEP-Zugangsdaten sind beschädigt. Bitte neu eingeben.",
            format!("Deserialisierung fehlgeschlagen: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn credentials(username: &str) -> ZepStoredCredentials {
        ZepStoredCredentials {
            username: username.to_string(),
            password: "geheim".to_string(),
        }
    }

    #[test]
    fn a_second_read_is_served_from_memory() {
        let cache = CredentialCache::default();
        let reads = AtomicUsize::new(0);
        let read = || {
            reads.fetch_add(1, Ordering::SeqCst);
            Ok(credentials("admin"))
        };

        cache.get_or_load(read).expect("first read should succeed");
        let second = cache.get_or_load(read).expect("second read should succeed");

        assert_eq!(second.username, "admin");
        assert_eq!(reads.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn a_failed_read_is_not_cached() {
        let cache = CredentialCache::default();

        let failed = cache.get_or_load(|| {
            Err(ZepError::new(
                ZepErrorCode::MissingCredentials,
                "Keine ZEP-Zugangsdaten hinterlegt.",
                "Kein Keychain-Eintrag",
            ))
        });
        let retried = cache.get_or_load(|| Ok(credentials("admin")));

        assert!(failed.is_err());
        assert_eq!(retried.expect("retry should succeed").username, "admin");
    }

    #[test]
    fn a_saved_value_replaces_the_cached_one() {
        let cache = CredentialCache::default();
        cache
            .get_or_load(|| Ok(credentials("alt")))
            .expect("first read should succeed");

        cache.store(credentials("neu"));
        let read = cache
            .get_or_load(|| Ok(credentials("aus dem Keychain")))
            .expect("read should succeed");

        assert_eq!(read.username, "neu");
    }
}
