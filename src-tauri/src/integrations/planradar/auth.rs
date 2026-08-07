use super::client::PlanradarApiClient;
use super::projects::{list_projects_core, PlanradarListProjectsInput};
use super::shared::{
    delete_api_token, load_store_or_error, normalize_base_url, peek_api_token, save_store_or_error,
    store_api_token, PlanradarApiError, PlanradarApiErrorCode, PreviousToken,
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

/// Stores the API token in the OS keychain and the non-secret base URL plus Customer ID in the
/// local config store.
/// The write order and rollback below keep those two stores from drifting apart.
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

    // Probe before persisting: a single-record list authenticates the token and exercises the
    // Customer ID path segment.
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

    // Load the store before touching the keychain so a store-read failure cannot orphan a token.
    let mut store = load_store_or_error(app.clone())?;
    let previous_token = peek_api_token();

    store_api_token(api_token)?;

    store.api_endpoints.planradar_base_url = base_url;
    store.api_endpoints.planradar_customer_id = customer_id.clone();
    if let Err(error) = save_store_or_error(app, store) {
        // On an unknown previous token, leave the keychain alone rather than risk deleting a
        // token that is still valid.
        let _ = match &previous_token {
            PreviousToken::Present(previous) => store_api_token(previous),
            PreviousToken::Absent => delete_api_token(),
            PreviousToken::Unknown => Ok(()),
        };
        return Err(error);
    }

    Ok(PlanradarConnectionStatus {
        // Not re-read from the keychain: a transient read error must not fail a persisted connect.
        has_token: true,
        customer_id,
    })
}

/// A raw "project not found" or "token invalid" is confusing in a credentials dialog; a wrong
/// Customer ID also surfaces as a 404 there.
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
