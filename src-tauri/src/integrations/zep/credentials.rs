use super::types::{ZepError, ZepErrorCode, ZepStoredCredentials};
use crate::secret_manager;

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
    )
}

pub(crate) fn load_zep_credentials_from_keychain() -> Result<ZepStoredCredentials, ZepError> {
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
