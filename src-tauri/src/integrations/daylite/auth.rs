use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::shared::{
    adopt_daylite_tokens, normalize_base_url, update_store_or_error, DayliteRefreshTokenRequest,
    DayliteTokenState, DayliteTokenSyncStatus,
};
use crate::error::AppError;
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;

#[tauri::command]
#[specta::specta]
pub async fn daylite_connect_refresh_token(
    app: tauri::AppHandle,
    request: DayliteRefreshTokenRequest,
) -> Result<DayliteTokenSyncStatus, AppError> {
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
) -> Result<DayliteTokenSyncStatus, AppError> {
    let base_url = normalize_base_url(&request.base_url)?;
    let client = DayliteApiClient::new(&base_url)?.with_telemetry(&app);

    let token_state = adopt_daylite_tokens(|| {
        mint_and_save_base_url(&client, request.refresh_token, || {
            update_store_or_error(&app, |store| {
                store.api_endpoints.daylite_base_url = base_url
            })
        })
    })
    .await?;

    Ok(DayliteTokenSyncStatus {
        has_access_token: !token_state.access_token.is_empty(),
        has_refresh_token: !token_state.refresh_token.is_empty(),
    })
}

/// Saved before the session adopts the tokens, so they are never paired with the previous base URL.
async fn mint_and_save_base_url(
    client: &DayliteApiClient,
    refresh_token: String,
    save_base_url: impl FnOnce() -> Result<(), AppError>,
) -> Result<DayliteTokenState, AppError> {
    let minted = refresh_tokens(client, refresh_token).await?;
    save_base_url()?;
    Ok(minted)
}

#[cfg(test)]
mod tests {
    use super::mint_and_save_base_url;
    use crate::error::{AppError, AppErrorCode};
    use crate::integrations::daylite::test_support::{mock_client, mock_response};
    use std::cell::Cell;

    const REFRESHED: &str = r#"{"access_token":"at-2","refresh_token":"rt-2","expires_in":3600}"#;

    #[tokio::test]
    async fn saves_the_base_url_once_the_token_is_minted() {
        let (client, _) = mock_client(vec![Ok(mock_response(200, REFRESHED))]);
        let saved = Cell::new(false);

        let minted = mint_and_save_base_url(&client, "rt".to_string(), || {
            saved.set(true);
            Ok(())
        })
        .await
        .expect("connect should succeed");

        assert_eq!(minted.refresh_token, "rt-2");
        assert!(saved.get());
    }

    #[tokio::test]
    async fn keeps_the_base_url_when_minting_fails() {
        let (client, _) = mock_client(vec![Ok(mock_response(401, r#"{"error":"nope"}"#))]);

        let error = mint_and_save_base_url(&client, "rt".to_string(), || {
            unreachable!("nothing was minted")
        })
        .await
        .expect_err("a rejected refresh token should fail");

        assert_eq!(error.code, AppErrorCode::TokenRefreshFailed);
    }

    #[tokio::test]
    async fn surfaces_a_failed_base_url_save() {
        let (client, _) = mock_client(vec![Ok(mock_response(200, REFRESHED))]);

        let error = mint_and_save_base_url(&client, "rt".to_string(), || {
            Err(AppError::new(
                AppErrorCode::InvalidConfiguration,
                "Speichern fehlgeschlagen",
                "store",
            ))
        })
        .await
        .expect_err("a failed save should surface");

        assert_eq!(error.code, AppErrorCode::InvalidConfiguration);
    }
}
