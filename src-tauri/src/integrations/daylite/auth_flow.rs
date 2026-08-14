use super::client::DayliteApiClient;
use super::client::DayliteHttpMethod;
use super::client::DayliteHttpRequest;
use super::shared::{
    current_epoch_ms, missing_token_error, normalize_http_error, parse_json_body,
    parse_success_json_body, truncate_for_log, DayliteApiError, DayliteApiErrorCode,
    DayliteTokenState,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;

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
        .send_request(DayliteHttpRequest {
            query: vec![("refresh_token".to_string(), refresh_token)],
            ..DayliteHttpRequest::new(DayliteHttpMethod::Get, "/personal_token/refresh_token")
        })
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
        return Err(DayliteApiError::new(
            DayliteApiErrorCode::TokenRefreshFailed,
            Some(response.status),
            "Das Daylite-Access-Token konnte nicht erneuert werden.",
            format!(
                "Refresh-Antwort enthält ein leeres access_token Feld. body={}",
                truncate_for_log(&response.body)
            ),
        ));
    }

    if refreshed_refresh_token.is_empty() {
        return Err(DayliteApiError::new(
            DayliteApiErrorCode::TokenRefreshFailed,
            Some(response.status),
            "Das Daylite-Refresh-Token konnte nicht erneuert werden.",
            format!(
                "Refresh-Antwort enthält ein leeres refresh_token Feld. body={}",
                truncate_for_log(&response.body)
            ),
        ));
    }

    if parsed_refresh.expires_in == 0 {
        return Err(DayliteApiError::new(
            DayliteApiErrorCode::TokenRefreshFailed,
            Some(response.status),
            "Die Ablaufzeit des Daylite-Access-Tokens ist ungültig.",
            format!(
                "Refresh-Antwort enthält expires_in=0. body={}",
                truncate_for_log(&response.body)
            ),
        ));
    }

    let now_ms = current_epoch_ms()?;
    let expires_at_ms = now_ms.saturating_add(parsed_refresh.expires_in.saturating_mul(1_000));

    Ok(DayliteTokenState {
        access_token,
        refresh_token: refreshed_refresh_token,
        access_token_expires_at_ms: Some(expires_at_ms),
    })
}

/// For endpoints that may return 204 No Content: verifies 2xx, ignores the body.
pub(super) async fn send_authenticated_request(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    mut request: DayliteHttpRequest,
) -> Result<(), DayliteApiError> {
    let path = request.path.clone();
    request.access_token = Some(token_state.access_token);
    let response = client.send_request(request).await?;
    if !(200..300).contains(&response.status) {
        return Err(normalize_http_error(response.status, &response.body, &path));
    }
    Ok(())
}

pub(super) async fn send_authenticated_json<T: DeserializeOwned>(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    mut request: DayliteHttpRequest,
) -> Result<T, DayliteApiError> {
    let path = request.path.clone();
    request.access_token = Some(token_state.access_token);
    let response = client.send_request(request).await?;

    parse_success_json_body::<T>(response.status, &response.body, &path)
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
        .map_err(|error| {
            DayliteApiError::new(
                DayliteApiErrorCode::TokenRefreshFailed,
                error.http_status,
                "Die Daylite-Token-Antwort konnte nicht verarbeitet werden.",
                error.technical_message,
            )
        })
}

#[cfg(test)]
mod tests {
    use super::{refresh_tokens, send_authenticated_json};
    use crate::integrations::daylite::client::{
        DayliteApiClient, DayliteHttpMethod, DayliteHttpRequest,
    };
    use crate::integrations::daylite::shared::{DayliteApiErrorCode, DayliteTokenState};
    use crate::integrations::daylite::test_support::{mock_client, mock_response, token_state};
    use serde::Deserialize;

    #[tokio::test]
    async fn refresh_tokens_rejects_blank_refresh_token() {
        let client =
            DayliteApiClient::new("https://daylite.example").expect("client should be created");

        let error = refresh_tokens(&client, "   ".to_string())
            .await
            .expect_err("blank refresh token should fail");

        assert_eq!(error.code, DayliteApiErrorCode::MissingToken);
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct AuthFlowFixture {
        value: String,
    }

    #[tokio::test]
    async fn send_authenticated_json_uses_existing_access_token_and_parses_payload() {
        let (client, transport) = mock_client(vec![Ok(mock_response(200, r#"{"value":"ok"}"#))]);

        let data = send_authenticated_json::<AuthFlowFixture>(
            &client,
            token_state("existing-access-token", "refresh-token"),
            DayliteHttpRequest {
                query: vec![("full-records".to_string(), "true".to_string())],
                ..DayliteHttpRequest::new(DayliteHttpMethod::Post, "/projects/_search")
            },
        )
        .await
        .expect("request should succeed");

        assert_eq!(
            data,
            AuthFlowFixture {
                value: "ok".to_string(),
            }
        );
        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].path, "/projects/_search");
        assert_eq!(
            requests[0].access_token,
            Some("existing-access-token".to_string())
        );
    }

    #[tokio::test]
    async fn send_authenticated_json_does_not_rotate_an_expired_token() {
        let (client, transport) = mock_client(vec![Ok(mock_response(401, r#"{"error":"nope"}"#))]);

        let error = send_authenticated_json::<AuthFlowFixture>(
            &client,
            DayliteTokenState {
                access_token: "expired-access-token".to_string(),
                refresh_token: "refresh-token".to_string(),
                access_token_expires_at_ms: Some(0),
            },
            DayliteHttpRequest::new(DayliteHttpMethod::Get, "/contacts/100"),
        )
        .await
        .expect_err("an expired token should be rejected, not rotated");

        assert_eq!(error.code, DayliteApiErrorCode::Unauthorized);

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].path, "/contacts/100");
    }

    #[tokio::test]
    async fn refresh_tokens_returns_error_on_non_2xx_status() {
        let (client, _) = mock_client(vec![Ok(mock_response(401, r#"{"error":"unauthorized"}"#))]);

        let error = refresh_tokens(&client, "valid-refresh-token".to_string())
            .await
            .expect_err("non-2xx refresh should fail");

        assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
        assert_eq!(error.http_status, Some(401));
    }

    #[tokio::test]
    async fn refresh_tokens_returns_error_on_malformed_json() {
        let (client, _) = mock_client(vec![Ok(mock_response(200, "this is not valid json"))]);

        let error = refresh_tokens(&client, "valid-refresh-token".to_string())
            .await
            .expect_err("malformed JSON refresh should fail");

        assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
    }

    #[tokio::test]
    async fn refresh_tokens_returns_error_on_empty_access_token() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"access_token":" ","refresh_token":"rt","expires_in":3600}"#,
        ))]);

        let error = refresh_tokens(&client, "valid-refresh-token".to_string())
            .await
            .expect_err("empty access_token should fail");

        assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
        assert!(error.technical_message.contains("access_token"));
    }

    #[tokio::test]
    async fn refresh_tokens_returns_error_on_empty_refresh_token_in_response() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"access_token":"at","refresh_token":"","expires_in":3600}"#,
        ))]);

        let error = refresh_tokens(&client, "valid-refresh-token".to_string())
            .await
            .expect_err("empty refresh_token in response should fail");

        assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
        assert!(error.technical_message.contains("refresh_token"));
    }

    #[tokio::test]
    async fn refresh_tokens_returns_error_on_zero_expires_in() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"access_token":"at","refresh_token":"rt","expires_in":0}"#,
        ))]);

        let error = refresh_tokens(&client, "valid-refresh-token".to_string())
            .await
            .expect_err("expires_in=0 should fail");

        assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
        assert!(error.technical_message.contains("expires_in=0"));
    }

    #[tokio::test]
    async fn send_authenticated_json_returns_error_on_non_2xx_response() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            500,
            r#"{"error":"internal server error"}"#,
        ))]);

        let error = send_authenticated_json::<AuthFlowFixture>(
            &client,
            token_state("valid-token", "refresh"),
            DayliteHttpRequest::new(DayliteHttpMethod::Get, "/projects/123"),
        )
        .await
        .expect_err("non-2xx response should fail");

        assert_eq!(error.code, DayliteApiErrorCode::ServerError);
        assert_eq!(error.http_status, Some(500));
    }

    #[tokio::test]
    async fn send_authenticated_json_returns_error_on_invalid_json_response() {
        let (client, _) = mock_client(vec![Ok(mock_response(200, "not valid json at all"))]);

        let error = send_authenticated_json::<AuthFlowFixture>(
            &client,
            token_state("valid-token", "refresh"),
            DayliteHttpRequest::new(DayliteHttpMethod::Get, "/contacts/100"),
        )
        .await
        .expect_err("invalid JSON response should fail");

        assert_eq!(error.code, DayliteApiErrorCode::InvalidResponse);
    }

    #[tokio::test]
    async fn refresh_tokens_replays_vcr_cassette() {
        let client = DayliteApiClient::with_replay_cassette("daylite-refresh-tokens.json")
            .expect("replay client should be created");

        let token_state = refresh_tokens(&client, "dummy-refresh-token".to_string())
            .await
            .expect("refresh should replay from cassette");

        assert_eq!(token_state.access_token, "replayed-access-token");
        assert_eq!(token_state.refresh_token, "replayed-refresh-token");
        assert!(token_state.access_token_expires_at_ms.is_some());
    }
}
