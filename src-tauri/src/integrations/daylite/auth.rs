use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::shared::{
    adopt_daylite_tokens, load_store_or_error, normalize_base_url, save_store_or_error,
    DayliteApiError, DayliteRefreshTokenRequest, DayliteTokenSyncStatus,
};
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

async fn daylite_connect_refresh_token_inner(
    app: tauri::AppHandle,
    request: DayliteRefreshTokenRequest,
) -> Result<DayliteTokenSyncStatus, DayliteApiError> {
    let base_url = normalize_base_url(&request.base_url)?;
    let client = DayliteApiClient::new(&base_url)?.with_telemetry(&app);

    let token_state =
        adopt_daylite_tokens(|| refresh_tokens(&client, request.refresh_token)).await?;

    let mut store = load_store_or_error(app.clone())?;
    store.api_endpoints.daylite_base_url = base_url;
    save_store_or_error(app, store)?;

    Ok(DayliteTokenSyncStatus {
        has_access_token: !token_state.access_token.is_empty(),
        has_refresh_token: !token_state.refresh_token.is_empty(),
    })
}
