use super::contacts::{DayliteContactSummary, DayliteContactUrl};
use super::projects::DayliteProjectSummary;
use super::shared::{
    build_limit_query, current_epoch_ms, missing_token_error, normalize_base_url,
    normalize_http_error, should_refresh_access_token, truncate_for_log, DayliteApiError,
    DayliteApiErrorCode, DayliteApiResponse, DayliteSearchResult, DayliteTokenState,
};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::header::AUTHORIZATION;

pub(super) struct DayliteApiClient {
    transport: Arc<dyn DayliteHttpTransport>,
}

impl DayliteApiClient {
    pub(super) fn new(base_url: &str) -> Result<Self, DayliteApiError> {
        let transport = ReqwestTransport::new(base_url)?;
        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    #[cfg(test)]
    fn with_transport(transport: Arc<dyn DayliteHttpTransport>) -> Self {
        Self { transport }
    }

    pub(super) async fn exchange_refresh_token(
        &self,
        refresh_token: String,
    ) -> Result<DayliteTokenState, DayliteApiError> {
        self.refresh_tokens(refresh_token).await
    }

    pub(super) async fn list_projects(
        &self,
        token_state: DayliteTokenState,
    ) -> Result<DayliteApiResponse<Vec<DayliteProjectSummary>>, DayliteApiError> {
        let search_response = self
            .execute_json_request::<DayliteSearchResult<DayliteProjectSummary>>(
                DayliteHttpMethod::Post,
                "/projects/_search",
                vec![("full-records".to_string(), "true".to_string())],
                Some(json!({})),
                token_state,
            )
            .await?;

        Ok(DayliteApiResponse {
            data: search_response.data.results,
            token_state: search_response.token_state,
        })
    }

    pub(super) async fn search_projects(
        &self,
        token_state: DayliteTokenState,
        search_term: &str,
        limit: Option<u16>,
    ) -> Result<DayliteApiResponse<DayliteSearchResult<DayliteProjectSummary>>, DayliteApiError>
    {
        self.execute_json_request(
            DayliteHttpMethod::Post,
            "/projects/_search",
            build_limit_query(limit),
            Some(json!({
                "name": {
                    "contains": search_term
                }
            })),
            token_state,
        )
        .await
    }

    pub(super) async fn list_contacts(
        &self,
        token_state: DayliteTokenState,
    ) -> Result<DayliteApiResponse<Vec<DayliteContactSummary>>, DayliteApiError> {
        let search_response = self
            .execute_json_request::<DayliteSearchResult<DayliteContactSummary>>(
                DayliteHttpMethod::Post,
                "/contacts/_search",
                vec![("full-records".to_string(), "true".to_string())],
                Some(json!({
                    "category": {
                        "equal": "Monteur"
                    }
                })),
                token_state,
            )
            .await?;

        Ok(DayliteApiResponse {
            data: search_response.data.results,
            token_state: search_response.token_state,
        })
    }

    pub(super) async fn search_contacts(
        &self,
        token_state: DayliteTokenState,
        search_term: &str,
        limit: Option<u16>,
    ) -> Result<DayliteApiResponse<DayliteSearchResult<DayliteContactSummary>>, DayliteApiError>
    {
        let mut query = build_limit_query(limit);
        query.push(("full-records".to_string(), "true".to_string()));

        self.execute_json_request(
            DayliteHttpMethod::Post,
            "/contacts/_search",
            query,
            Some(json!({
                "full_name": {
                    "contains": search_term
                }
            })),
            token_state,
        )
        .await
    }

    pub(super) async fn update_contact_ical_urls(
        &self,
        token_state: DayliteTokenState,
        contact_reference: &str,
        primary_ical_url: &str,
        absence_ical_url: &str,
    ) -> Result<DayliteApiResponse<DayliteContactSummary>, DayliteApiError> {
        let contact_id = parse_contact_id(contact_reference)?;
        let contact_path = format!("/contacts/{contact_id}");
        let current_contact = self
            .execute_json_request::<DayliteContactSummary>(
                DayliteHttpMethod::Get,
                &contact_path,
                Vec::new(),
                None,
                token_state,
            )
            .await?;

        let merged_urls = merge_contact_ical_urls(
            current_contact.data.urls,
            primary_ical_url,
            absence_ical_url,
        );

        self.execute_json_request(
            DayliteHttpMethod::Patch,
            &contact_path,
            Vec::new(),
            Some(json!({
                "urls": merged_urls,
            })),
            current_contact.token_state,
        )
        .await
    }

    async fn execute_json_request<T: DeserializeOwned>(
        &self,
        method: DayliteHttpMethod,
        path: &str,
        query: Vec<(String, String)>,
        body: Option<Value>,
        mut token_state: DayliteTokenState,
    ) -> Result<DayliteApiResponse<T>, DayliteApiError> {
        if token_state.access_token.trim().is_empty() && token_state.refresh_token.trim().is_empty()
        {
            return Err(missing_token_error(
                "Es sind keine Daylite-Zugangsdaten hinterlegt. Bitte ein Refresh-Token hinterlegen.",
                "Weder Access- noch Refresh-Token sind vorhanden.",
            ));
        }

        let now_ms = current_epoch_ms()?;
        if should_refresh_access_token(&token_state, now_ms) {
            token_state = self
                .refresh_tokens(token_state.refresh_token.clone())
                .await?;
        }

        let response = self
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

        let raw_json =
            serde_json::from_str::<Value>(&response.body).map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::InvalidResponse,
                http_status: Some(response.status),
                user_message: "Die Antwort von Daylite konnte nicht verarbeitet werden."
                    .to_string(),
                technical_message: format!(
                    "JSON-Parsing für {path} fehlgeschlagen: {error}; body={}",
                    truncate_for_log(&response.body)
                ),
            })?;
        log_contact_payload_shape(path, &raw_json);

        let data = serde_json::from_value::<T>(raw_json).map_err(|error| DayliteApiError {
            code: DayliteApiErrorCode::InvalidResponse,
            http_status: Some(response.status),
            user_message: "Die Antwort von Daylite konnte nicht verarbeitet werden.".to_string(),
            technical_message: format!(
                "JSON-Deserialisierung für {path} fehlgeschlagen: {error}; body={}",
                truncate_for_log(&response.body)
            ),
        })?;

        Ok(DayliteApiResponse { data, token_state })
    }

    async fn refresh_tokens(
        &self,
        refresh_token: String,
    ) -> Result<DayliteTokenState, DayliteApiError> {
        if refresh_token.trim().is_empty() {
            return Err(missing_token_error(
                "Das Daylite-Refresh-Token fehlt. Bitte Refresh-Token hinterlegen.",
                "Refresh-Token ist leer.",
            ));
        }

        let response = self
            .send_request(
                DayliteHttpMethod::Get,
                "/personal_token/refresh_token",
                vec![("refresh_token".to_string(), refresh_token.clone())],
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

        let parsed_refresh = parse_refresh_response_body(&response)?;
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

    async fn send_request(
        &self,
        method: DayliteHttpMethod,
        path: &str,
        query: Vec<(String, String)>,
        body: Option<Value>,
        access_token: Option<String>,
    ) -> Result<DayliteHttpResponse, DayliteApiError> {
        self.transport
            .send(DayliteHttpRequest {
                method,
                path: path.to_string(),
                query,
                body,
                access_token,
            })
            .await
    }
}

fn log_contact_payload_shape(path: &str, payload: &Value) {
    if !path.starts_with("/contacts") {
        return;
    }

    let Some(results) = payload.get("results").and_then(Value::as_array) else {
        if path.starts_with("/contacts/") {
            println!(
                "[daylite-contacts] raw contact payload path={path} sample={:?}",
                summarize_contact_payload(payload)
            );
        }
        return;
    };

    let sample = results
        .iter()
        .take(5)
        .map(summarize_contact_payload)
        .collect::<Vec<_>>();

    println!(
        "[daylite-contacts] raw contact payload path={path} loaded={} sample={sample:?}",
        results.len()
    );
}

fn summarize_contact_payload(
    contact: &Value,
) -> (
    Option<String>,
    Vec<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    usize,
) {
    let mut keys = contact
        .as_object()
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    keys.sort();

    (
        read_contact_string(contact, "self"),
        keys,
        read_contact_string(contact, "first_name")
            .or_else(|| read_contact_string(contact, "firstName")),
        read_contact_string(contact, "last_name")
            .or_else(|| read_contact_string(contact, "lastName")),
        read_contact_string(contact, "full_name")
            .or_else(|| read_contact_string(contact, "fullName")),
        read_contact_string(contact, "name"),
        read_contact_string(contact, "category"),
        contact.get("categories").is_some(),
        contact
            .get("urls")
            .and_then(Value::as_array)
            .map_or(0, std::vec::Vec::len),
    )
}

fn read_contact_string(contact: &Value, key: &str) -> Option<String> {
    contact.get(key).and_then(Value::as_str).map(str::to_string)
}

pub(super) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub(super) trait DayliteHttpTransport: Send + Sync {
    fn send<'a>(
        &'a self,
        request: DayliteHttpRequest,
    ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DayliteHttpMethod {
    Get,
    Post,
    Patch,
}

#[derive(Debug, Clone)]
pub(super) struct DayliteHttpRequest {
    pub method: DayliteHttpMethod,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub access_token: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct DayliteHttpResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone)]
struct ReqwestTransport {
    base_url: String,
    http_client: reqwest::Client,
}

impl ReqwestTransport {
    fn new(base_url: &str) -> Result<Self, DayliteApiError> {
        let normalized_base_url = normalize_base_url(base_url)?;
        let http_client = reqwest::Client::builder()
            .build()
            .map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: None,
                user_message: "Die Verbindung zu Daylite konnte nicht aufgebaut werden."
                    .to_string(),
                technical_message: format!("HTTP-Client konnte nicht erstellt werden: {error}"),
            })?;

        Ok(Self {
            base_url: normalized_base_url,
            http_client,
        })
    }
}

impl DayliteHttpTransport for ReqwestTransport {
    fn send<'a>(
        &'a self,
        request: DayliteHttpRequest,
    ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>> {
        Box::pin(async move {
            let mut url = reqwest::Url::parse(&format!("{}{}", self.base_url, request.path))
                .map_err(|error| DayliteApiError {
                    code: DayliteApiErrorCode::InvalidConfiguration,
                    http_status: None,
                    user_message: "Die Daylite-URL ist ungültig konfiguriert.".to_string(),
                    technical_message: format!("URL konnte nicht geparst werden: {error}"),
                })?;

            {
                let mut query_pairs = url.query_pairs_mut();
                for (key, value) in &request.query {
                    query_pairs.append_pair(key, value);
                }
            }

            let mut builder = match request.method {
                DayliteHttpMethod::Get => self.http_client.get(url),
                DayliteHttpMethod::Post => self.http_client.post(url),
                DayliteHttpMethod::Patch => self.http_client.patch(url),
            };

            if let Some(access_token) = request.access_token {
                if !access_token.trim().is_empty() {
                    builder = builder.header(AUTHORIZATION, format!("Bearer {access_token}"));
                }
            }

            if let Some(body) = request.body {
                builder = builder
                    .header("content-type", "application/json")
                    .body(body.to_string());
            }

            let response = builder.send().await.map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: None,
                user_message: "Die Anfrage an Daylite ist fehlgeschlagen.".to_string(),
                technical_message: format!("Netzwerkfehler bei {}: {error}", request.path),
            })?;

            let status = response.status().as_u16();
            let body = response.text().await.map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: Some(status),
                user_message: "Die Antwort von Daylite konnte nicht gelesen werden.".to_string(),
                technical_message: format!("Antworttext konnte nicht gelesen werden: {error}"),
            })?;

            Ok(DayliteHttpResponse { status, body })
        })
    }
}

#[derive(Debug, Deserialize)]
struct DayliteRefreshTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

fn parse_refresh_response_body(
    response: &DayliteHttpResponse,
) -> Result<DayliteRefreshTokenResponse, DayliteApiError> {
    serde_json::from_str::<DayliteRefreshTokenResponse>(&response.body).map_err(|error| {
        DayliteApiError {
            code: DayliteApiErrorCode::TokenRefreshFailed,
            http_status: Some(response.status),
            user_message: "Die Daylite-Token-Antwort konnte nicht verarbeitet werden.".to_string(),
            technical_message: format!(
                "Ungültige Refresh-Antwort. Erwartet wurden access_token, refresh_token, expires_in. error={error}; body={}",
                truncate_for_log(&response.body)
            ),
        }
    })
}

fn parse_contact_id(contact_reference: &str) -> Result<u64, DayliteApiError> {
    let trimmed_reference = contact_reference.trim();
    let contact_id_raw = trimmed_reference.rsplit('/').next().unwrap_or_default();

    contact_id_raw
        .parse::<u64>()
        .map_err(|error| DayliteApiError {
            code: DayliteApiErrorCode::InvalidResponse,
            http_status: None,
            user_message: "Die Daylite-Kontaktreferenz ist ungültig.".to_string(),
            technical_message: format!("Ungültige Kontaktreferenz `{trimmed_reference}`: {error}"),
        })
}

fn merge_contact_ical_urls(
    existing_urls: Vec<DayliteContactUrl>,
    primary_ical_url: &str,
    absence_ical_url: &str,
) -> Vec<DayliteContactUrl> {
    let mut merged_urls = existing_urls
        .into_iter()
        .filter(|url| {
            let Some(label) = normalize_label(url.label.as_deref()) else {
                return true;
            };

            !is_primary_ical_label(&label) && !is_absence_ical_label(&label)
        })
        .collect::<Vec<_>>();

    let normalized_primary_ical_url = normalize_non_empty(primary_ical_url);
    if let Some(primary_url) = normalized_primary_ical_url {
        merged_urls.push(DayliteContactUrl {
            label: Some("Einsatz iCal".to_string()),
            url: Some(primary_url.to_string()),
            note: None,
        });
    }

    let normalized_absence_ical_url = normalize_non_empty(absence_ical_url);
    if let Some(absence_url) = normalized_absence_ical_url {
        merged_urls.push(DayliteContactUrl {
            label: Some("Abwesenheit iCal".to_string()),
            url: Some(absence_url.to_string()),
            note: None,
        });
    }

    merged_urls
}

fn is_primary_ical_label(label: &str) -> bool {
    label.contains("einsatz") || label.contains("termine")
}

fn is_absence_ical_label(label: &str) -> bool {
    label.contains("abwesenheit") || label.contains("fehlzeiten")
}

fn normalize_label(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
}

fn normalize_non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[test]
    fn list_projects_returns_typed_data_for_200_response() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                vec![],
                r#"{"results":[{"self":"/v1/projects/1000","name":"Projekt Alpha","status":"in_progress","category":"Überfällig","keywords":["Aufträge","Vorbereitung"]}]}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_projects(DayliteTokenState {
                    access_token: "access-1".to_string(),
                    refresh_token: "refresh-1".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect("request should succeed");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.data[0].reference, "/v1/projects/1000");
            assert_eq!(result.data[0].name, "Projekt Alpha");
            assert_eq!(result.data[0].status, Some("in_progress".to_string()));
            assert_eq!(transport.requests().len(), 1);
            assert_eq!(transport.requests()[0].method, DayliteHttpMethod::Post);
            assert_eq!(transport.requests()[0].path, "/projects/_search");
            assert_eq!(
                transport.requests()[0].query,
                vec![("full-records".to_string(), "true".to_string())]
            );
            assert_eq!(transport.requests()[0].body, Some(json!({})));
        });
    }

    #[test]
    fn list_projects_returns_normalized_401_error() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                401,
                vec![],
                r#"{"error":"unauthorized"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = client
                .list_projects(DayliteTokenState {
                    access_token: "invalid-token".to_string(),
                    refresh_token: String::new(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect_err("request should fail");

            assert_eq!(error.code, DayliteApiErrorCode::Unauthorized);
            assert_eq!(error.http_status, Some(401));
            assert!(!error.user_message.is_empty());
            assert!(!error.technical_message.is_empty());
        });
    }

    #[test]
    fn list_projects_returns_normalized_429_error() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                429,
                vec![],
                r#"{"error":"rate_limited"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = client
                .list_projects(DayliteTokenState {
                    access_token: "access-1".to_string(),
                    refresh_token: "refresh-1".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect_err("request should fail");

            assert_eq!(error.code, DayliteApiErrorCode::RateLimited);
            assert_eq!(error.http_status, Some(429));
        });
    }

    #[test]
    fn list_projects_returns_normalized_500_error() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                500,
                vec![],
                r#"{"error":"server_error"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = client
                .list_projects(DayliteTokenState {
                    access_token: "access-1".to_string(),
                    refresh_token: "refresh-1".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect_err("request should fail");

            assert_eq!(error.code, DayliteApiErrorCode::ServerError);
            assert_eq!(error.http_status, Some(500));
        });
    }

    #[test]
    fn list_projects_returns_invalid_response_error_for_malformed_payload() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                vec![],
                r#"{"self":"/v1/projects/1000","name":"Projekt Alpha"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = client
                .list_projects(DayliteTokenState {
                    access_token: "access-1".to_string(),
                    refresh_token: "refresh-1".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect_err("request should fail because payload is not a search object");

            assert_eq!(error.code, DayliteApiErrorCode::InvalidResponse);
            assert_eq!(error.http_status, Some(200));
        });
    }

    #[test]
    fn list_projects_refreshes_before_request_when_access_token_is_expired() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(
                    200,
                    vec![],
                    r#"{"access_token":"refreshed-access-token","expires_in":3600,"token_type":"Bearer","scope":"daylite:read daylite:write","refresh_token":"refreshed-refresh-token"}"#,
                )),
                Ok(mock_response(
                    200,
                    vec![],
                    r#"{"results":[{"self":"/v1/projects/2000","name":"Projekt Beta"}]}"#,
                )),
            ]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_projects(DayliteTokenState {
                    access_token: "expired-access-token".to_string(),
                    refresh_token: "initial-refresh-token".to_string(),
                    access_token_expires_at_ms: Some(0),
                })
                .await
                .expect("request should succeed after refresh");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.token_state.access_token, "refreshed-access-token");
            assert_eq!(result.token_state.refresh_token, "refreshed-refresh-token");
            assert!(result.token_state.access_token_expires_at_ms.is_some());

            let requests = transport.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].path, "/personal_token/refresh_token");
            assert_eq!(
                requests[0].query,
                vec![(
                    "refresh_token".to_string(),
                    "initial-refresh-token".to_string()
                )]
            );
            assert_eq!(requests[1].path, "/projects/_search");
            assert_eq!(
                requests[1].access_token,
                Some("refreshed-access-token".to_string())
            );
        });
    }

    #[test]
    fn list_projects_refreshes_when_access_token_is_missing() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(
                    200,
                    vec![],
                    r#"{"access_token":"new-access-token","expires_in":3600,"token_type":"Bearer","scope":"daylite:read daylite:write","refresh_token":"new-refresh-token"}"#,
                )),
                Ok(mock_response(
                    200,
                    vec![],
                    r#"{"results":[{"self":"/v1/projects/4000","name":"Projekt Delta"}]}"#,
                )),
            ]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_projects(DayliteTokenState {
                    access_token: String::new(),
                    refresh_token: "initial-refresh-token".to_string(),
                    access_token_expires_at_ms: None,
                })
                .await
                .expect("request should succeed after oauth-style token refresh");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.token_state.access_token, "new-access-token");
            assert_eq!(result.token_state.refresh_token, "new-refresh-token");
            assert!(result.token_state.access_token_expires_at_ms.is_some());

            let requests = transport.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].path, "/personal_token/refresh_token");
            assert_eq!(requests[1].path, "/projects/_search");
            assert_eq!(
                requests[1].access_token,
                Some("new-access-token".to_string())
            );
        });
    }

    #[test]
    fn list_projects_uses_existing_access_token_when_it_is_still_valid() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                vec![],
                r#"{"results":[{"self":"/v1/projects/3000","name":"Projekt Gamma"}]}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_projects(DayliteTokenState {
                    access_token: "active-access-token".to_string(),
                    refresh_token: "initial-refresh-token".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect("request should succeed with existing access token");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.token_state.access_token, "active-access-token");
            assert_eq!(result.token_state.refresh_token, "initial-refresh-token");

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/projects/_search");
            assert_eq!(
                requests[0].access_token,
                Some("active-access-token".to_string())
            );
        });
    }

    #[test]
    fn list_projects_fails_refresh_when_token_fields_are_not_snake_case() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                vec![],
                r#"{"accessToken":"new-access-token","refreshToken":"new-refresh-token","expires_in":3600}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = client
                .list_projects(DayliteTokenState {
                    access_token: String::new(),
                    refresh_token: "initial-refresh-token".to_string(),
                    access_token_expires_at_ms: None,
                })
                .await
                .expect_err("request should fail for unsupported token key names");

            assert_eq!(error.code, DayliteApiErrorCode::TokenRefreshFailed);
        });
    }

    #[test]
    fn list_projects_does_not_refresh_on_401_when_access_token_not_near_expiry() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                401,
                vec![],
                r#"{"error":"unauthorized"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let error = client
                .list_projects(DayliteTokenState {
                    access_token: "invalid-access-token".to_string(),
                    refresh_token: "refresh-token-1".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect_err("request should fail with unauthorized");

            assert_eq!(error.code, DayliteApiErrorCode::Unauthorized);

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/projects/_search");
        });
    }

    #[test]
    fn search_contacts_returns_typed_search_result() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                vec![],
                r#"{"results":[{"self":"/v1/contacts/100","first_name":"Max","last_name":"Mustermann"}],"next":"/v1/contacts/_search?start=101"}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .search_contacts(
                    DayliteTokenState {
                        access_token: "access-1".to_string(),
                        refresh_token: "refresh-1".to_string(),
                        access_token_expires_at_ms: Some(u64::MAX),
                    },
                    "Max",
                    Some(10),
                )
                .await
                .expect("search should succeed");

            assert_eq!(result.data.results.len(), 1);
            assert_eq!(result.data.results[0].reference, "/v1/contacts/100");
            assert_eq!(
                result.data.next,
                Some("/v1/contacts/_search?start=101".to_string())
            );

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/contacts/_search");
            assert_eq!(
                requests[0].query,
                vec![
                    ("limit".to_string(), "10".to_string()),
                    ("full-records".to_string(), "true".to_string())
                ]
            );
            assert_eq!(
                requests[0].body,
                Some(json!({
                    "full_name": {
                        "contains": "Max"
                    }
                }))
            );
        });
    }

    #[test]
    fn list_contacts_uses_full_records_search_endpoint() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                vec![],
                r#"{"results":[{"self":"/v1/contacts/100","full_name":"Max Mustermann","category":"Monteur","urls":[{"label":"Einsatz iCal","url":"https://example.com/max-primary.ics"}]}]}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_contacts(DayliteTokenState {
                    access_token: "access-1".to_string(),
                    refresh_token: "refresh-1".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                })
                .await
                .expect("list contacts should succeed");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.data[0].reference, "/v1/contacts/100");
            assert_eq!(result.data[0].category, Some("Monteur".to_string()));

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].method, DayliteHttpMethod::Post);
            assert_eq!(requests[0].path, "/contacts/_search");
            assert_eq!(
                requests[0].query,
                vec![("full-records".to_string(), "true".to_string())]
            );
            assert_eq!(
                requests[0].body,
                Some(json!({
                    "category": {
                        "equal": "Monteur"
                    }
                }))
            );
        });
    }

    #[test]
    fn update_contact_ical_urls_patches_contact_urls_only() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(
                    200,
                    vec![],
                    r#"{"self":"/v1/contacts/100","full_name":"Max Mustermann","category":"Monteur","urls":[{"label":"Website","url":"https://example.com"},{"label":"FR-Fehlzeiten","url":"https://example.com/old-absence.ics"}]}"#,
                )),
                Ok(mock_response(
                    200,
                    vec![],
                    r#"{"self":"/v1/contacts/100","full_name":"Max Mustermann","category":"Monteur","urls":[{"label":"Website","url":"https://example.com"},{"label":"Einsatz iCal","url":"https://example.com/new-primary.ics"},{"label":"Abwesenheit iCal","url":"https://example.com/new-absence.ics"}]}"#,
                )),
            ]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .update_contact_ical_urls(
                    DayliteTokenState {
                        access_token: "access-1".to_string(),
                        refresh_token: "refresh-1".to_string(),
                        access_token_expires_at_ms: Some(u64::MAX),
                    },
                    "/v1/contacts/100",
                    "https://example.com/new-primary.ics",
                    "https://example.com/new-absence.ics",
                )
                .await
                .expect("update contact should succeed");

            assert_eq!(result.data.reference, "/v1/contacts/100");

            let requests = transport.requests();
            assert_eq!(requests.len(), 2);
            assert_eq!(requests[0].method, DayliteHttpMethod::Get);
            assert_eq!(requests[0].path, "/contacts/100");
            assert_eq!(requests[1].method, DayliteHttpMethod::Patch);
            assert_eq!(requests[1].path, "/contacts/100");
            assert_eq!(
                requests[1].body,
                Some(json!({
                    "urls": [
                        {
                            "label": "Website",
                            "url": "https://example.com"
                        },
                        {
                            "label": "Einsatz iCal",
                            "url": "https://example.com/new-primary.ics"
                        },
                        {
                            "label": "Abwesenheit iCal",
                            "url": "https://example.com/new-absence.ics"
                        }
                    ]
                }))
            );
        });
    }

    #[test]
    fn returns_missing_token_error_when_no_tokens_are_available() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(Vec::new());
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let error = client
                .list_projects(DayliteTokenState::default())
                .await
                .expect_err("request should fail");

            assert_eq!(error.code, DayliteApiErrorCode::MissingToken);
            assert_eq!(error.http_status, None);
        });
    }

    #[test]
    fn should_refresh_access_token_when_less_than_ten_seconds_remain() {
        let token_state = DayliteTokenState {
            access_token: "access-token-1".to_string(),
            refresh_token: "refresh-token-1".to_string(),
            access_token_expires_at_ms: Some(9_999),
        };

        assert!(should_refresh_access_token(&token_state, 0));
    }

    #[test]
    fn should_not_refresh_access_token_when_more_than_ten_seconds_remain() {
        let token_state = DayliteTokenState {
            access_token: "access-token-1".to_string(),
            refresh_token: "refresh-token-1".to_string(),
            access_token_expires_at_ms: Some(10_001),
        };

        assert!(!should_refresh_access_token(&token_state, 0));
    }

    #[test]
    fn build_limit_query_returns_empty_query_without_limit() {
        assert_eq!(build_limit_query(None), Vec::new());
    }

    #[test]
    fn build_limit_query_sets_limit_parameter_when_present() {
        assert_eq!(
            build_limit_query(Some(25)),
            vec![("limit".to_string(), "25".to_string())]
        );
    }

    #[derive(Clone)]
    struct MockTransport {
        responses: Arc<Mutex<VecDeque<Result<DayliteHttpResponse, DayliteApiError>>>>,
        requests: Arc<Mutex<Vec<DayliteHttpRequest>>>,
    }

    impl MockTransport {
        fn new(responses: Vec<Result<DayliteHttpResponse, DayliteApiError>>) -> Self {
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
        ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>> {
            Box::pin(async move {
                self.requests
                    .lock()
                    .expect("request lock should succeed")
                    .push(request);

                self.responses
                    .lock()
                    .expect("response lock should succeed")
                    .pop_front()
                    .unwrap_or_else(|| {
                        Err(DayliteApiError {
                            code: DayliteApiErrorCode::InvalidResponse,
                            http_status: None,
                            user_message:
                                "Test-Mock hat keine weitere Antwort für die Anfrage hinterlegt."
                                    .to_string(),
                            technical_message: "MockTransport response queue was empty during test"
                                .to_string(),
                        })
                    })
            })
        }
    }

    fn mock_response(
        status: u16,
        _headers: Vec<(String, String)>,
        body: &str,
    ) -> DayliteHttpResponse {
        DayliteHttpResponse {
            status,
            body: body.to_string(),
        }
    }
}
