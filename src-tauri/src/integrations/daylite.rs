use super::local_store::{self, LocalStore};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use specta::Type;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::header::AUTHORIZATION;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteTokenState {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteTokenSyncStatus {
    pub has_access_token: bool,
    pub has_refresh_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteProjectSummary {
    #[serde(rename = "self")]
    pub reference: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteContactSummary {
    #[serde(rename = "self")]
    pub reference: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteSearchResult<T> {
    pub results: Vec<T>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteApiResponse<T> {
    pub data: T,
    pub token_state: DayliteTokenState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteApiError {
    pub code: DayliteApiErrorCode,
    pub http_status: Option<u16>,
    pub user_message: String,
    pub technical_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DayliteApiErrorCode {
    Unauthorized,
    RateLimited,
    ServerError,
    MissingToken,
    InvalidConfiguration,
    RequestFailed,
    InvalidResponse,
    TokenRefreshFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DaylitePersonalTokenRequest {
    pub base_url: String,
    pub personal_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteSearchInput {
    pub search_term: String,
    pub limit: Option<u16>,
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_connect_personal_token(
    app: tauri::AppHandle,
    request: DaylitePersonalTokenRequest,
) -> Result<DayliteTokenSyncStatus, DayliteApiError> {
    let base_url = normalize_base_url(&request.base_url)?;
    let client = DayliteApiClient::new(&base_url)?;
    let token_state = client
        .exchange_personal_token(request.personal_token)
        .await?;

    let mut store = load_store_or_error(app.clone())?;
    store.api_endpoints.daylite_base_url = base_url;
    store_daylite_tokens(&mut store, &token_state);
    save_store_or_error(app, store)?;

    Ok(DayliteTokenSyncStatus {
        has_access_token: !token_state.access_token.is_empty(),
        has_refresh_token: !token_state.refresh_token.is_empty(),
    })
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_projects(
    app: tauri::AppHandle,
) -> Result<Vec<DayliteProjectSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = daylite_client_from_store(&store)?;
    let token_state = load_daylite_tokens(&store);

    let response = client.list_projects(token_state).await?;
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_search_projects(
    app: tauri::AppHandle,
    input: DayliteSearchInput,
) -> Result<DayliteSearchResult<DayliteProjectSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = daylite_client_from_store(&store)?;
    let token_state = load_daylite_tokens(&store);

    let response = client
        .search_projects(token_state, &input.search_term, input.limit)
        .await?;
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<DayliteContactSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = daylite_client_from_store(&store)?;
    let token_state = load_daylite_tokens(&store);

    let response = client.list_contacts(token_state).await?;
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_search_contacts(
    app: tauri::AppHandle,
    input: DayliteSearchInput,
) -> Result<DayliteSearchResult<DayliteContactSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = daylite_client_from_store(&store)?;
    let token_state = load_daylite_tokens(&store);

    let response = client
        .search_contacts(token_state, &input.search_term, input.limit)
        .await?;
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}

pub struct DayliteApiClient {
    transport: Arc<dyn DayliteHttpTransport>,
}

impl DayliteApiClient {
    pub fn new(base_url: &str) -> Result<Self, DayliteApiError> {
        let transport = ReqwestTransport::new(base_url)?;
        Ok(Self {
            transport: Arc::new(transport),
        })
    }

    #[cfg(test)]
    fn with_transport(transport: Arc<dyn DayliteHttpTransport>) -> Self {
        Self { transport }
    }

    pub async fn exchange_personal_token(
        &self,
        personal_token: String,
    ) -> Result<DayliteTokenState, DayliteApiError> {
        self.refresh_tokens(personal_token).await
    }

    pub async fn list_projects(
        &self,
        token_state: DayliteTokenState,
    ) -> Result<DayliteApiResponse<Vec<DayliteProjectSummary>>, DayliteApiError> {
        self.execute_json_request(
            DayliteHttpMethod::Get,
            "/projects",
            Vec::new(),
            None,
            token_state,
        )
        .await
    }

    pub async fn search_projects(
        &self,
        token_state: DayliteTokenState,
        search_term: &str,
        limit: Option<u16>,
    ) -> Result<DayliteApiResponse<DayliteSearchResult<DayliteProjectSummary>>, DayliteApiError>
    {
        let mut query = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit".to_string(), limit.to_string()));
        }

        self.execute_json_request(
            DayliteHttpMethod::Post,
            "/projects/_search",
            query,
            Some(json!({
                "name": {
                    "contains": search_term
                }
            })),
            token_state,
        )
        .await
    }

    pub async fn list_contacts(
        &self,
        token_state: DayliteTokenState,
    ) -> Result<DayliteApiResponse<Vec<DayliteContactSummary>>, DayliteApiError> {
        self.execute_json_request(
            DayliteHttpMethod::Get,
            "/contacts",
            Vec::new(),
            None,
            token_state,
        )
        .await
    }

    pub async fn search_contacts(
        &self,
        token_state: DayliteTokenState,
        search_term: &str,
        limit: Option<u16>,
    ) -> Result<DayliteApiResponse<DayliteSearchResult<DayliteContactSummary>>, DayliteApiError>
    {
        let mut query = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit".to_string(), limit.to_string()));
        }

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
                "Es ist kein Daylite-Token hinterlegt. Bitte zuerst ein Personal Token verbinden.",
                "Weder Access- noch Refresh-Token sind vorhanden.",
            ));
        }

        if token_state.access_token.trim().is_empty() {
            token_state = self
                .refresh_tokens(token_state.refresh_token.clone())
                .await?;
        }

        let mut response = self
            .send_request(
                method,
                path,
                query.clone(),
                body.clone(),
                Some(token_state.access_token.clone()),
            )
            .await?;

        apply_rotated_tokens(&response, &mut token_state);

        if response.status == 401 {
            if token_state.refresh_token.trim().is_empty() {
                return Err(normalize_http_error(response.status, &response.body, path));
            }

            token_state = self
                .refresh_tokens(token_state.refresh_token.clone())
                .await?;
            response = self
                .send_request(
                    method,
                    path,
                    query,
                    body,
                    Some(token_state.access_token.clone()),
                )
                .await?;
            apply_rotated_tokens(&response, &mut token_state);
        }

        if !(200..300).contains(&response.status) {
            return Err(normalize_http_error(response.status, &response.body, path));
        }

        let data = serde_json::from_str::<T>(&response.body).map_err(|error| DayliteApiError {
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
                "Das Daylite-Refresh-Token fehlt. Bitte Personal Token erneut verbinden.",
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

        let access_token = extract_access_token(&response).ok_or_else(|| DayliteApiError {
            code: DayliteApiErrorCode::TokenRefreshFailed,
            http_status: Some(response.status),
            user_message: "Das Daylite-Access-Token konnte nicht erneuert werden.".to_string(),
            technical_message: format!(
                "Kein Access-Token in Refresh-Antwort gefunden. headers={:?}, body={}",
                response.headers,
                truncate_for_log(&response.body)
            ),
        })?;

        let refreshed_refresh_token =
            extract_refresh_token(&response).unwrap_or_else(|| refresh_token.clone());

        Ok(DayliteTokenState {
            access_token,
            refresh_token: refreshed_refresh_token,
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

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait DayliteHttpTransport: Send + Sync {
    fn send<'a>(
        &'a self,
        request: DayliteHttpRequest,
    ) -> BoxFuture<'a, Result<DayliteHttpResponse, DayliteApiError>>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DayliteHttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone)]
struct DayliteHttpRequest {
    method: DayliteHttpMethod,
    path: String,
    query: Vec<(String, String)>,
    body: Option<Value>,
    access_token: Option<String>,
}

#[derive(Debug, Clone)]
struct DayliteHttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
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
            let mut headers = HashMap::new();
            for (key, value) in response.headers() {
                headers.insert(
                    key.as_str().to_ascii_lowercase(),
                    value.to_str().unwrap_or_default().to_string(),
                );
            }

            let body = response.text().await.map_err(|error| DayliteApiError {
                code: DayliteApiErrorCode::RequestFailed,
                http_status: Some(status),
                user_message: "Die Antwort von Daylite konnte nicht gelesen werden.".to_string(),
                technical_message: format!("Antworttext konnte nicht gelesen werden: {error}"),
            })?;

            Ok(DayliteHttpResponse {
                status,
                headers,
                body,
            })
        })
    }
}

fn normalize_http_error(status: u16, body: &str, path: &str) -> DayliteApiError {
    let (code, user_message) = if status == 401 {
        (
            DayliteApiErrorCode::Unauthorized,
            "Die Daylite-Anmeldung ist abgelaufen oder ungültig.",
        )
    } else if status == 429 {
        (
            DayliteApiErrorCode::RateLimited,
            "Daylite hat zu viele Anfragen erhalten. Bitte kurz warten und erneut versuchen.",
        )
    } else if (500..=599).contains(&status) {
        (
            DayliteApiErrorCode::ServerError,
            "Daylite ist aktuell nicht erreichbar.",
        )
    } else {
        (
            DayliteApiErrorCode::RequestFailed,
            "Die Daten konnten nicht von Daylite geladen werden.",
        )
    };

    DayliteApiError {
        code,
        http_status: Some(status),
        user_message: user_message.to_string(),
        technical_message: format!(
            "Daylite request failed for {path} with status={status}; body={}",
            truncate_for_log(body)
        ),
    }
}

fn missing_token_error(user_message: &str, technical_message: &str) -> DayliteApiError {
    DayliteApiError {
        code: DayliteApiErrorCode::MissingToken,
        http_status: None,
        user_message: user_message.to_string(),
        technical_message: technical_message.to_string(),
    }
}

fn apply_rotated_tokens(response: &DayliteHttpResponse, token_state: &mut DayliteTokenState) {
    if let Some(access_token) = extract_access_token(response) {
        token_state.access_token = access_token;
    }
    if let Some(refresh_token) = extract_refresh_token(response) {
        token_state.refresh_token = refresh_token;
    }
}

fn extract_access_token(response: &DayliteHttpResponse) -> Option<String> {
    extract_header_token(response, &["authorization", "access-token", "access_token"])
        .map(|token| normalize_access_token(&token))
        .filter(|token| !token.is_empty())
        .or_else(|| {
            extract_token_from_json(
                &response.body,
                &["authorization", "access_token", "access-token", "token"],
            )
            .map(|token| normalize_access_token(&token))
            .filter(|token| !token.is_empty())
        })
}

fn extract_refresh_token(response: &DayliteHttpResponse) -> Option<String> {
    extract_header_token(
        response,
        &["refresh-token", "refresh_token", "x-refresh-token"],
    )
    .filter(|token| !token.trim().is_empty())
    .or_else(|| {
        extract_token_from_json(&response.body, &["refresh_token", "refresh-token"])
            .filter(|token| !token.trim().is_empty())
    })
}

fn extract_header_token(response: &DayliteHttpResponse, key_candidates: &[&str]) -> Option<String> {
    key_candidates.iter().find_map(|key| {
        response
            .headers
            .get(*key)
            .map(|value| value.trim().to_string())
    })
}

fn extract_token_from_json(body: &str, key_candidates: &[&str]) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(body).ok()?;
    find_json_key_value(&parsed, key_candidates)
}

fn find_json_key_value(value: &Value, key_candidates: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for (key, current_value) in map {
                if key_candidates
                    .iter()
                    .any(|candidate| key.eq_ignore_ascii_case(candidate))
                {
                    if let Some(token) = current_value.as_str() {
                        return Some(token.to_string());
                    }
                }

                if let Some(found) = find_json_key_value(current_value, key_candidates) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(values) => values
            .iter()
            .find_map(|item| find_json_key_value(item, key_candidates)),
        _ => None,
    }
}

fn normalize_access_token(token_value: &str) -> String {
    let trimmed = token_value.trim();
    if trimmed.len() >= 7 && trimmed[..7].eq_ignore_ascii_case("bearer ") {
        return trimmed[7..].trim().to_string();
    }

    trimmed.to_string()
}

fn truncate_for_log(value: &str) -> String {
    let limit = 400;
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut truncated = value.chars().take(limit).collect::<String>();
    truncated.push_str("...");
    truncated
}

fn normalize_base_url(base_url: &str) -> Result<String, DayliteApiError> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(DayliteApiError {
            code: DayliteApiErrorCode::InvalidConfiguration,
            http_status: None,
            user_message: "Die Daylite-URL ist nicht konfiguriert.".to_string(),
            technical_message: "Leere dayliteBaseUrl-Konfiguration".to_string(),
        });
    }

    Ok(trimmed.to_string())
}

fn daylite_client_from_store(store: &LocalStore) -> Result<DayliteApiClient, DayliteApiError> {
    DayliteApiClient::new(&store.api_endpoints.daylite_base_url)
}

fn load_daylite_tokens(store: &LocalStore) -> DayliteTokenState {
    DayliteTokenState {
        access_token: store.token_references.daylite_access_token.clone(),
        refresh_token: store.token_references.daylite_refresh_token.clone(),
    }
}

fn store_daylite_tokens(store: &mut LocalStore, token_state: &DayliteTokenState) {
    store.token_references.daylite_access_token = token_state.access_token.clone();
    store.token_references.daylite_refresh_token = token_state.refresh_token.clone();
}

fn load_store_or_error(app: tauri::AppHandle) -> Result<LocalStore, DayliteApiError> {
    local_store::load_local_store(app).map_err(map_store_error)
}

fn save_store_or_error(app: tauri::AppHandle, store: LocalStore) -> Result<(), DayliteApiError> {
    local_store::save_local_store(app, store).map_err(map_store_error)
}

fn map_store_error(error: local_store::StoreError) -> DayliteApiError {
    DayliteApiError {
        code: DayliteApiErrorCode::InvalidConfiguration,
        http_status: None,
        user_message: error.user_message,
        technical_message: error.technical_message,
    }
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
                r#"[{"self":"/v1/projects/1000","name":"Projekt Alpha"}]"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_projects(DayliteTokenState {
                    access_token: "access-1".to_string(),
                    refresh_token: "refresh-1".to_string(),
                })
                .await
                .expect("request should succeed");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.data[0].reference, "/v1/projects/1000");
            assert_eq!(result.data[0].name, "Projekt Alpha");
            assert_eq!(transport.requests().len(), 1);
            assert_eq!(transport.requests()[0].path, "/projects");
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
                })
                .await
                .expect_err("request should fail");

            assert_eq!(error.code, DayliteApiErrorCode::ServerError);
            assert_eq!(error.http_status, Some(500));
        });
    }

    #[test]
    fn list_projects_uses_personal_token_refresh_flow_and_rotates_tokens() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(401, vec![], r#"{"error":"unauthorized"}"#)),
                Ok(mock_response(
                    200,
                    vec![
                        (
                            "authorization".to_string(),
                            "Bearer refreshed-access-token".to_string(),
                        ),
                        (
                            "refresh-token".to_string(),
                            "refreshed-refresh-token".to_string(),
                        ),
                    ],
                    r#"{"result":"ok"}"#,
                )),
                Ok(mock_response(
                    200,
                    vec![],
                    r#"[{"self":"/v1/projects/2000","name":"Projekt Beta"}]"#,
                )),
            ]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let result = client
                .list_projects(DayliteTokenState {
                    access_token: "expired-access-token".to_string(),
                    refresh_token: "initial-refresh-token".to_string(),
                })
                .await
                .expect("request should succeed after refresh");

            assert_eq!(result.data.len(), 1);
            assert_eq!(result.token_state.access_token, "refreshed-access-token");
            assert_eq!(result.token_state.refresh_token, "refreshed-refresh-token");

            let requests = transport.requests();
            assert_eq!(requests.len(), 3);
            assert_eq!(requests[0].path, "/projects");
            assert_eq!(requests[1].path, "/personal_token/refresh_token");
            assert_eq!(
                requests[1].query,
                vec![(
                    "refresh_token".to_string(),
                    "initial-refresh-token".to_string()
                )]
            );
            assert_eq!(requests[2].path, "/projects");
            assert_eq!(
                requests[2].access_token,
                Some("refreshed-access-token".to_string())
            );
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
                vec![("limit".to_string(), "10".to_string())]
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
        headers: Vec<(String, String)>,
        body: &str,
    ) -> DayliteHttpResponse {
        DayliteHttpResponse {
            status,
            headers: headers.into_iter().collect(),
            body: body.to_string(),
        }
    }
}
