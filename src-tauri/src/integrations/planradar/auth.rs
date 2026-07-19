use super::client::PlanradarApiClient;
use super::projects::{list_projects_core, PlanradarListProjectsInput};
use super::shared::{
    delete_api_token, load_store_or_error, normalize_base_url, peek_api_token, save_store_or_error,
    store_api_token, PlanradarApiError, PlanradarApiErrorCode,
};
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

/// Stores the user-provided Planradar credentials: the API token goes into the OS keychain
/// (via the secret manager), while the non-secret base URL and Customer ID go into the local
/// config store. There is no OAuth or token rotation; the token is used verbatim per request.
///
/// The credentials are verified against the live API with a lightweight authenticated probe
/// (a one-record project list) before anything is persisted, so an invalid token or wrong
/// Customer ID fails fast instead of silently succeeding here and erroring on the first real
/// call. Persistence is ordered so the keychain and store never end up out of sync: the config
/// store is loaded before the token is written, the previous token is snapshotted, and if the
/// store write fails the previous token is restored (or removed if there was none).
#[tauri::command]
#[specta::specta]
pub async fn planradar_connect(
    app: tauri::AppHandle,
    request: PlanradarConnectRequest,
) -> Result<PlanradarConnectionStatus, PlanradarApiError> {
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

    let api_token = request.api_token.trim();
    if api_token.is_empty() {
        return Err(PlanradarApiError::new(
            PlanradarApiErrorCode::MissingToken,
            None,
            "Das Planradar-Token darf nicht leer sein.",
            "planradar_connect mit leerem api_token aufgerufen",
        ));
    }

    // Verify the credentials before persisting anything: a single-record project list both
    // authenticates the token and exercises the Customer ID path segment. Probe failures are
    // remapped to a connect-specific message because a raw "project not found" (404) or
    // "token invalid" (401) is confusing in a credentials dialog.
    let client = PlanradarApiClient::new(&base_url)?;
    list_projects_core(
        &client,
        api_token,
        &customer_id,
        &PlanradarListProjectsInput {
            pagesize: Some(1),
            ..PlanradarListProjectsInput::default()
        },
    )
    .await
    .map_err(remap_probe_error)?;

    // Load the store before touching the keychain so a store-read failure cannot leave an orphan
    // token, and snapshot the current token so we can restore it if the store write fails.
    let mut store = load_store_or_error(app.clone())?;
    let previous_token = peek_api_token();

    store_api_token(api_token)?;

    store.api_endpoints.planradar_base_url = base_url;
    store.api_endpoints.planradar_customer_id = customer_id.clone();
    if let Err(error) = save_store_or_error(app, store) {
        // Restore the previous token (or remove the new one) so a store-write failure cannot
        // leave the keychain and config store out of sync.
        let _ = match &previous_token {
            Some(previous) => store_api_token(previous),
            None => delete_api_token(),
        };
        return Err(error);
    }

    Ok(PlanradarConnectionStatus {
        // The token was just written successfully, so it is present without re-querying the
        // keychain (a transient read error there must not fail an already-persisted connect).
        has_token: true,
        customer_id,
    })
}

/// Remaps probe failures during connect into a message that makes sense in a credentials dialog.
/// An invalid token (401/403) or wrong Customer ID (often surfaced as 404) should point the user
/// at their credentials rather than at a missing project or expired session.
fn remap_probe_error(error: PlanradarApiError) -> PlanradarApiError {
    match error.code {
        PlanradarApiErrorCode::Unauthorized
        | PlanradarApiErrorCode::NotFound
        | PlanradarApiErrorCode::MissingCustomerId => PlanradarApiError::new(
            error.code,
            error.http_status,
            "Verbindung fehlgeschlagen. Bitte Customer ID und API-Token prüfen.",
            error.technical_message,
        ),
        _ => error,
    }
}
