use super::shared::{
    has_api_token, load_store_or_error, normalize_base_url, save_store_or_error, store_api_token,
    PlanradarApiError, PlanradarApiErrorCode,
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

    store_api_token(api_token)?;

    let mut store = load_store_or_error(app.clone())?;
    store.api_endpoints.planradar_base_url = base_url;
    store.api_endpoints.planradar_customer_id = customer_id.clone();
    save_store_or_error(app, store)?;

    Ok(PlanradarConnectionStatus {
        has_token: has_api_token()?,
        customer_id,
    })
}
