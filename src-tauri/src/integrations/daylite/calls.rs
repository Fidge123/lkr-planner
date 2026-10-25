use super::auth_flow::refresh_tokens;
use super::client::DayliteApiClient;
use super::shared::{
    current_epoch_ms, load_daylite_tokens, load_store_or_error, missing_token_error,
    DayliteTokenState,
};
use super::token_session::{token_session, TokenLease};
use crate::error::{AppError, AppErrorCode};

async fn lease_tokens(client: &DayliteApiClient) -> Result<TokenLease, AppError> {
    let session = token_session();
    if session.is_empty() {
        session.seed(load_daylite_tokens()?);
    }

    session
        .lease(current_epoch_ms()?, |current| {
            rotate_tokens(client, current)
        })
        .await
}

async fn rotate_tokens(
    client: &DayliteApiClient,
    current: DayliteTokenState,
) -> Result<DayliteTokenState, AppError> {
    if current.access_token.trim().is_empty() && current.refresh_token.trim().is_empty() {
        return Err(missing_token_error(
            "Es sind keine Daylite-Zugangsdaten hinterlegt. Bitte ein Refresh-Token hinterlegen.",
            "Weder Access- noch Refresh-Token sind vorhanden.",
        ));
    }

    refresh_tokens(client, current.refresh_token).await
}

/// Retries once on rejection: a token revoked server-side defeats expiry-driven rotation.
pub(super) async fn with_daylite_tokens<T, F, Fut>(
    client: &DayliteApiClient,
    operation: F,
) -> Result<T, AppError>
where
    F: Fn(DayliteTokenState) -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
{
    let lease = lease_tokens(client).await?;
    retry_once_on_unauthorized(lease.tokens.clone(), operation, || async {
        let renewed = token_session()
            .renew(&lease, |current| rotate_tokens(client, current))
            .await?;
        Ok(renewed.tokens)
    })
    .await
}

async fn retry_once_on_unauthorized<T, F, Fut, R, RFut>(
    tokens: DayliteTokenState,
    operation: F,
    renew: R,
) -> Result<T, AppError>
where
    F: Fn(DayliteTokenState) -> Fut,
    Fut: std::future::Future<Output = Result<T, AppError>>,
    R: FnOnce() -> RFut,
    RFut: std::future::Future<Output = Result<DayliteTokenState, AppError>>,
{
    match operation(tokens).await {
        Err(error) if error.code == AppErrorCode::Unauthorized => operation(renew().await?).await,
        result => result,
    }
}

/// For operations that cannot be replayed, such as one that has already mutated the store.
pub(super) async fn with_daylite_tokens_once<Fut>(
    client: &DayliteApiClient,
    operation: impl FnOnce(DayliteTokenState) -> Fut,
) -> Result<(), AppError>
where
    Fut: std::future::Future<Output = Result<(), AppError>>,
{
    let lease = lease_tokens(client).await?;
    operation(lease.tokens).await
}

/// Only for read-only bodies; mutating commands manage the local store themselves.
pub(super) async fn run_daylite_command<T>(
    app: tauri::AppHandle,
    operation: impl AsyncFn(&DayliteApiClient, DayliteTokenState) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?.with_telemetry(&app);
    with_daylite_tokens(&client, |tokens| operation(&client, tokens)).await
}

pub(super) async fn adopt_daylite_tokens<F, Fut>(mint: F) -> Result<DayliteTokenState, AppError>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<DayliteTokenState, AppError>>,
{
    token_session().adopt(mint).await
}

/// Must run before the process exits, including the restart that installs an update.
pub async fn close_daylite_token_session() -> Result<(), AppError> {
    token_session().close().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn tokens(access: &str) -> DayliteTokenState {
        DayliteTokenState {
            access_token: access.to_string(),
            refresh_token: "rt".to_string(),
            access_token_expires_at_ms: Some(900_000),
        }
    }

    fn unauthorized() -> AppError {
        AppError::new(AppErrorCode::Unauthorized, "abgelaufen", "401").with_http_status(Some(401))
    }

    #[tokio::test]
    async fn a_successful_call_does_not_renew() {
        let renewals = AtomicUsize::new(0);

        let value = retry_once_on_unauthorized(
            tokens("at"),
            |_| async { Ok("ok") },
            || async {
                renewals.fetch_add(1, Ordering::SeqCst);
                Ok(tokens("renewed"))
            },
        )
        .await
        .expect("call should succeed");

        assert_eq!(value, "ok");
        assert_eq!(renewals.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn a_rejected_token_is_renewed_and_the_call_replayed() {
        let attempts = AtomicUsize::new(0);
        let seen = std::sync::Mutex::new(Vec::new());

        let value = retry_once_on_unauthorized(
            tokens("at"),
            |current| {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                seen.lock().unwrap().push(current.access_token.clone());
                async move {
                    if attempt == 0 {
                        return Err(unauthorized());
                    }
                    Ok("ok")
                }
            },
            || async { Ok(tokens("renewed")) },
        )
        .await
        .expect("call should succeed after renewal");

        assert_eq!(value, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
        assert_eq!(*seen.lock().unwrap(), vec!["at", "renewed"]);
    }

    #[tokio::test]
    async fn a_second_rejection_is_not_retried_again() {
        let attempts = AtomicUsize::new(0);

        let error = retry_once_on_unauthorized(
            tokens("at"),
            |_| {
                attempts.fetch_add(1, Ordering::SeqCst);
                async { Err::<&str, _>(unauthorized()) }
            },
            || async { Ok(tokens("renewed")) },
        )
        .await
        .expect_err("a second rejection should surface");

        assert_eq!(error.code, AppErrorCode::Unauthorized);
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn an_error_that_is_not_a_rejection_does_not_renew() {
        let renewals = AtomicUsize::new(0);

        let error = retry_once_on_unauthorized(
            tokens("at"),
            |_| async {
                Err::<&str, _>(
                    AppError::new(AppErrorCode::RateLimited, "zu viele", "429")
                        .with_http_status(Some(429)),
                )
            },
            || async {
                renewals.fetch_add(1, Ordering::SeqCst);
                Ok(tokens("renewed"))
            },
        )
        .await
        .expect_err("the error should surface");

        assert_eq!(error.code, AppErrorCode::RateLimited);
        assert_eq!(renewals.load(Ordering::SeqCst), 0);
    }
}
