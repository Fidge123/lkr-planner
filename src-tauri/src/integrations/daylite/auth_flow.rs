use super::client::DayliteApiClient;
use super::client::DayliteHttpMethod;
use super::shared::{
    current_epoch_ms, missing_token_error, normalize_http_error, parse_json_body,
    parse_success_json_body, should_refresh_access_token, truncate_for_log, DayliteApiError,
    DayliteApiErrorCode, DayliteTokenState,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;

pub(super) async fn ensure_access_token(
    client: &DayliteApiClient,
    mut token_state: DayliteTokenState,
) -> Result<DayliteTokenState, DayliteApiError> {
    if token_state.access_token.trim().is_empty() && token_state.refresh_token.trim().is_empty() {
        return Err(missing_token_error(
            "Es sind keine Daylite-Zugangsdaten hinterlegt. Bitte ein Refresh-Token hinterlegen.",
            "Weder Access- noch Refresh-Token sind vorhanden.",
        ));
    }

    let now_ms = current_epoch_ms()?;
    if should_refresh_access_token(&token_state, now_ms) {
        token_state = refresh_tokens(client, token_state.refresh_token.clone()).await?;
    }

    Ok(token_state)
}

pub(super) async fn refresh_tokens(
    client: &DayliteApiClient,
    refresh_token: String,
) -> Result<DayliteTokenState, DayliteApiError> {
    if refresh_token.trim().is_empty() {
        return Err(missing_token_error(
            "Das Daylite-Refresh-Token fehlt. Bitte Refresh-Token hinterlegen.",
            "Refresh-Token ist leer.",
        ));
    }

    let response = client
        .send_request(
            DayliteHttpMethod::Get,
            "/personal_token/refresh_token",
            vec![("refresh_token".to_string(), refresh_token)],
            None,
            None,
        )
        .await?;

    if !(200..300).contains(&response.status) {
        let mut error = normalize_http_error(
            response.status,
            &response.body,
            "/personal_token/refresh_token",
        );
        error.code = DayliteApiErrorCode::TokenRefreshFailed;
        return Err(error);
    }

    let parsed_refresh = parse_refresh_response_body(response.status, &response.body)?;
    let access_token = parsed_refresh.access_token.trim().to_string();
    let refreshed_refresh_token = parsed_refresh.refresh_token.trim().to_string();

    if access_token.is_empty() {
        return Err(DayliteApiError {
            code: DayliteApiErrorCode::TokenRefreshFailed,
            http_status: Some(response.status),
            user_message: "Das Daylite-Access-Token konnte nicht erneuert werden.".to_string(),
            technical_message: format!(
                "Refresh-Antwort enthält ein leeres access_token Feld. body={}",
                truncate_for_log(&response.body)
            ),
        });
    }

    if refreshed_refresh_token.is_empty() {
        return Err(DayliteApiError {
            code: DayliteApiErrorCode::TokenRefreshFailed,
            http_status: Some(response.status),
            user_message: "Das Daylite-Refresh-Token konnte nicht erneuert werden.".to_string(),
            technical_message: format!(
                "Refresh-Antwort enthält ein leeres refresh_token Feld. body={}",
                truncate_for_log(&response.body)
            ),
        });
    }

    if parsed_refresh.expires_in == 0 {
        return Err(DayliteApiError {
            code: DayliteApiErrorCode::TokenRefreshFailed,
            http_status: Some(response.status),
            user_message: "Die Ablaufzeit des Daylite-Access-Tokens ist ungültig.".to_string(),
            technical_message: format!(
                "Refresh-Antwort enthält expires_in=0. body={}",
                truncate_for_log(&response.body)
            ),
        });
    }

    let now_ms = current_epoch_ms()?;
    let expires_at_ms = now_ms.saturating_add(parsed_refresh.expires_in.saturating_mul(1_000));

    Ok(DayliteTokenState {
        access_token,
        refresh_token: refreshed_refresh_token,
        access_token_expires_at_ms: Some(expires_at_ms),
    })
}

/// Send an authenticated request and verify success (2xx status), ignoring the response body.
/// Use for PATCH/PUT endpoints that may return 204 No Content instead of a JSON body.
pub(super) async fn send_authenticated_request(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    method: DayliteHttpMethod,
    path: &str,
    query: Vec<(String, String)>,
    body: Option<Value>,
) -> Result<DayliteTokenState, DayliteApiError> {
    let token_state = ensure_access_token(client, token_state).await?;
    let response = client
        .send_request(
            method,
            path,
            query,
            body,
            Some(token_state.access_token.clone()),
        )
        .await?;
    if !(200..300).contains(&response.status) {
        return Err(normalize_http_error(response.status, &response.body, path));
    }
    Ok(token_state)
}

pub(super) async fn send_authenticated_json<T: DeserializeOwned>(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    method: DayliteHttpMethod,
    path: &str,
    query: Vec<(String, String)>,
    body: Option<Value>,
) -> Result<(T, DayliteTokenState), DayliteApiError> {
    let token_state = ensure_access_token(client, token_state).await?;
    let response = client
        .send_request(
            method,
            path,
            query,
            body,
            Some(token_state.access_token.clone()),
        )
        .await?;
    let data = parse_success_json_body::<T>(response.status, &response.body, path)?;

    Ok((data, token_state))
}

#[derive(Debug, Deserialize)]
struct DayliteRefreshTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

fn parse_refresh_response_body(
    status: u16,
    body: &str,
) -> Result<DayliteRefreshTokenResponse, DayliteApiError> {
    parse_json_body::<DayliteRefreshTokenResponse>(status, body, "/personal_token/refresh_token")
        .map_err(|error| DayliteApiError {
            code: DayliteApiErrorCode::TokenRefreshFailed,
            http_status: error.http_status,
            user_message: "Die Daylite-Token-Antwort konnte nicht verarbeitet werden.".to_string(),
            technical_message: error.technical_message,
        })
}

#[cfg(test)]
mod tests {
    use super::{ensure_access_token, refresh_tokens, send_authenticated_json};
    use crate::integrations::daylite::client::{
        BoxFuture, DayliteApiClient, DayliteHttpMethod, DayliteHttpRequest, DayliteHttpResponse,
        DayliteHttpTransport,
    };
    use crate::integrations::daylite::shared::{DayliteApiErrorCode, DayliteTokenState};
    use serde::Deserialize;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[test]
    fn ensure_access_token_returns_missing_token_error_when_both_tokens_are_missing() {
        tauri::async_runtime::block_on(async {
            let client =
                DayliteApiClient::new("https://daylite.example").expect("client should be created");

            let error = ensure_access_token(&client, DayliteTokenState::default())
                .await
                .expect_err("missing tokens should fail");

            assert_eq!(error.code, DayliteApiErrorCode::MissingToken);
        });
    }

    #[test]
    fn ensure_access_token_keeps_existing_access_token_when_expiry_is_in_future() {
        tauri::async_runtime::block_on(async {
            let client =
                DayliteApiClient::new("https://daylite.example").expect("client should be created");
            let original_state = DayliteTokenState {
                access_token: "existing-access-token".to_string(),
                refresh_token: "refresh-token".to_string(),
                access_token_expires_at_ms: Some(u64::MAX),
            };

            let token_state = ensure_access_token(&client, original_state.clone())
                .await
                .expect("existing token should be reused");

            assert_eq!(token_state, original_state);
        });
    }

    #[test]
    fn refresh_tokens_rejects_blank_refresh_token() {
        tauri::async_runtime::block_on(async {
            let client =
                DayliteApiClient::new("https://daylite.example").expect("client should be created");

            let error = refresh_tokens(&client, "   ".to_string())
                .await
                .expect_err("blank refresh token should fail");

            assert_eq!(error.code, DayliteApiErrorCode::MissingToken);
        });
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct AuthFlowFixture {
        value: String,
    }

    #[test]
    fn send_authenticated_json_uses_existing_access_token_and_parses_payload() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(200, r#"{"value":"ok"}"#))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let (data, token_state) = send_authenticated_json::<AuthFlowFixture>(
                &client,
                DayliteTokenState {
                    access_token: "existing-access-token".to_string(),
                    refresh_token: "refresh-token".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
                DayliteHttpMethod::Post,
                "/projects/_search",
                vec![("full-records".to_string(), "true".to_string())],
                None,
            )
            .await
            .expect("request should succeed");

            assert_eq!(
                data,
                AuthFlowFixture {
                    value: "ok".to_string(),
                }
            );
            assert_eq!(token_state.access_token, "existing-access-token");

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/projects/_search");
            assert_eq!(
                requests[0].access_token,
                Some("existing-access-token".to_string())
            );
        });
    }

    #[test]
    fn send_authenticated_json_refreshes_before_request_when_access_token_is_missing() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(
                    200,
                    r#"{"access_token":"refreshed-access-token","refresh_token":"refreshed-refresh-token","expires_in":3600}"#,
                )),
                Ok(mock_response(200, r#"{"value":"ok"}"#)),
            ]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let (_, token_state) = send_authenticated_json::<AuthFlowFixture>(
                &client,
                DayliteTokenState {
                    access_token: String::new(),
                    refresh_token: "initial-refresh-token".to_string(),
                    access_token_expires_at_ms: None,
                },
                DayliteHttpMethod::Get,
                "/contacts/100",
                Vec::new(),
                None,
            )
            .await
            .expect("request should succeed after refresh");

            assert_eq!(token_state.access_token, "refreshed-access-token");
            assert_eq!(token_state.refresh_token, "refreshed-refresh-token");
            assert!(token_state.access_token_expires_at_ms.is_some());

            let requests = transport.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].path, "/personal_token/refresh_token");
            assert_eq!(requests[1].path, "/contacts/100");
            assert_eq!(
                requests[1].access_token,
                Some("refreshed-access-token".to_string())
            );
        });
    }

    #[test]
    fn refresh_tokens_returns_error_on_non_2xx_status() {
        tauri::async_runtime::block_on(async {
            let transport =
                MockTransport::new(vec![Ok(mock_response(401, r#"{"error":"unauthorized"}"#))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = refresh_tokens(&client, "valid-refresh-token".to_string())
                .await
                .expect_err("non-2xx refresh should fail");

            assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
            assert_eq!(error.http_status, Some(401));
        });
    }

    #[test]
    fn refresh_tokens_returns_error_on_malformed_json() {
        tauri::async_runtime::block_on(async {
            let transport =
                MockTransport::new(vec![Ok(mock_response(200, "this is not valid json"))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = refresh_tokens(&client, "valid-refresh-token".to_string())
                .await
                .expect_err("malformed JSON refresh should fail");

            assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
        });
    }

    #[test]
    fn refresh_tokens_returns_error_on_empty_access_token() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"access_token":" ","refresh_token":"rt","expires_in":3600}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = refresh_tokens(&client, "valid-refresh-token".to_string())
                .await
                .expect_err("empty access_token should fail");

            assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
            assert!(error.technical_message.contains("access_token"));
        });
    }

    #[test]
    fn refresh_tokens_returns_error_on_empty_refresh_token_in_response() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"access_token":"at","refresh_token":"","expires_in":3600}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = refresh_tokens(&client, "valid-refresh-token".to_string())
                .await
                .expect_err("empty refresh_token in response should fail");

            assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
            assert!(error.technical_message.contains("refresh_token"));
        });
    }

    #[test]
    fn refresh_tokens_returns_error_on_zero_expires_in() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"access_token":"at","refresh_token":"rt","expires_in":0}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = refresh_tokens(&client, "valid-refresh-token".to_string())
                .await
                .expect_err("expires_in=0 should fail");

            assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
            assert!(error.technical_message.contains("expires_in=0"));
        });
    }

    #[test]
    fn send_authenticated_json_returns_error_on_non_2xx_response() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                500,
                r#"{"error":"internal server error"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = send_authenticated_json::<AuthFlowFixture>(
                &client,
                DayliteTokenState {
                    access_token: "valid-token".to_string(),
                    refresh_token: "refresh".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
                DayliteHttpMethod::Get,
                "/projects/123",
                Vec::new(),
                None,
            )
            .await
            .expect_err("non-2xx response should fail");

            assert_eq!(error.code, DayliteApiErrorCode::ServerError);
            assert_eq!(error.http_status, Some(500));
        });
    }

    #[test]
    fn send_authenticated_json_returns_error_on_invalid_json_response() {
        tauri::async_runtime::block_on(async {
            let transport =
                MockTransport::new(vec![Ok(mock_response(200, "not valid json at all"))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = send_authenticated_json::<AuthFlowFixture>(
                &client,
                DayliteTokenState {
                    access_token: "valid-token".to_string(),
                    refresh_token: "refresh".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
                DayliteHttpMethod::Get,
                "/contacts/100",
                Vec::new(),
                None,
            )
            .await
            .expect_err("invalid JSON response should fail");

            assert_eq!(error.code, DayliteApiErrorCode::InvalidResponse);
        });
    }

    #[test]
    fn refresh_tokens_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = DayliteApiClient::with_replay_cassette("daylite-refresh-tokens.json")
                .expect("replay client should be created");

            let token_state = refresh_tokens(&client, "dummy-refresh-token".to_string())
                .await
                .expect("refresh should replay from cassette");

            assert_eq!(token_state.access_token, "replayed-access-token");
            assert_eq!(token_state.refresh_token, "replayed-refresh-token");
            assert!(token_state.access_token_expires_at_ms.is_some());
        });
    }

    #[derive(Clone)]
    struct MockTransport {
        responses: Arc<
            Mutex<
                VecDeque<
                    Result<
                        DayliteHttpResponse,
                        crate::integrations::daylite::shared::DayliteApiError,
                    >,
                >,
            >,
        >,
        requests: Arc<Mutex<Vec<DayliteHttpRequest>>>,
    }

    impl MockTransport {
        fn new(
            responses: Vec<
                Result<DayliteHttpResponse, crate::integrations::daylite::shared::DayliteApiError>,
            >,
        ) -> Self {
            Self {
                responses: Arc::new(Mutex::new(VecDeque::from(responses))),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn requests(&self) -> Vec<DayliteHttpRequest> {
            self.requests
                .lock()
                .expect("request lock should succeed")
                .clone()
        }
    }

    impl DayliteHttpTransport for MockTransport {
        fn send<'a>(
            &'a self,
            request: DayliteHttpRequest,
        ) -> BoxFuture<
            'a,
            Result<DayliteHttpResponse, crate::integrations::daylite::shared::DayliteApiError>,
        > {
            Box::pin(async move {
                self.requests
                    .lock()
                    .expect("request lock should succeed")
                    .push(request);

                self.responses
                    .lock()
                    .expect("response lock should succeed")
                    .pop_front()
                    .expect("mock should contain enough responses")
            })
        }
    }

    fn mock_response(status: u16, body: &str) -> DayliteHttpResponse {
        DayliteHttpResponse {
            status,
            body: body.to_string(),
        }
    }
}
