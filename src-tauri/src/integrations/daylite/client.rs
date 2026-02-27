use super::shared::{normalize_base_url, DayliteApiError, DayliteApiErrorCode};
use serde_json::Value;
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

    pub(super) async fn send_request(
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
