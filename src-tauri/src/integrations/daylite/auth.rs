use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;

#[tauri::command]
#[specta::specta]
pub async fn daylite_connect_refresh_token(
    app: tauri::AppHandle,
    request: DayliteRefreshTokenRequest,
) -> Result<DayliteTokenSyncStatus, DayliteApiError> {
    let handle = app.clone();
    observe(
        &handle,
        Operation::DayliteConnectRefreshToken,
        Integration::Daylite,
        daylite_connect_refresh_token_inner(app, request),
    )
    .await
}

use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::shared::{
    load_store_or_error, normalize_base_url, save_store_or_error, with_token_refresh_lock,
    DayliteApiError, DayliteRefreshTokenRequest, DayliteTokenSyncStatus,
};

async fn daylite_connect_refresh_token_inner(
    app: tauri::AppHandle,
    request: DayliteRefreshTokenRequest,
) -> Result<DayliteTokenSyncStatus, DayliteApiError> {
    let base_url = normalize_base_url(&request.base_url)?;
    let client = DayliteApiClient::new(&base_url)?.with_telemetry(&app);

    // Persist the freshly minted tokens under the same lock the other commands use, so a connect cannot interleave with a concurrent refresh. The existing tokens are ignored.
    let token_state = with_token_refresh_lock(|_existing| async move {
        let refreshed = refresh_tokens(&client, request.refresh_token).await?;
        Ok((refreshed.clone(), refreshed))
    })
    .await?;

    let mut store = load_store_or_error(app.clone())?;
    store.api_endpoints.daylite_base_url = base_url;
    save_store_or_error(app, store)?;

    Ok(DayliteTokenSyncStatus {
        has_access_token: !token_state.access_token.is_empty(),
        has_refresh_token: !token_state.refresh_token.is_empty(),
    })
}
