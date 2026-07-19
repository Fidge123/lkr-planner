use super::client::{PlanradarApiClient, PlanradarHttpMethod};
use super::shared::{
    load_api_token, load_config, load_store_or_error, parse_success_json_body, PlanradarApiError,
    PlanradarApiErrorCode, PlanradarConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use specta::Type;

/// Planradar project lifecycle status. The API encodes status as an integer where `1` is an
/// active project and `9` is an archived one (see the archive-project endpoint).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlanradarProjectStatus {
    Active,
    Archived,
}

/// Status code sent to the archive-project endpoint to unarchive (reactivate) a project.
const PLANRADAR_STATUS_ACTIVE: i64 = 1;
/// Status code that marks a project as archived.
const PLANRADAR_STATUS_ARCHIVED: i64 = 9;

impl PlanradarProjectStatus {
    fn from_api_status(status: i64) -> Self {
        if status == PLANRADAR_STATUS_ARCHIVED {
            Self::Archived
        } else {
            Self::Active
        }
    }
}

/// Frontend-facing Planradar project summary, normalized from the JSON:API `data` object.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarProject {
    pub id: String,
    pub name: String,
    pub status: PlanradarProjectStatus,
}

/// Pagination and sorting options for listing projects (`GET .../projects`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarListProjectsInput {
    #[serde(default)]
    pub sort: Option<String>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub pagesize: Option<u32>,
}

/// Attributes for creating a blank project (`POST .../projects`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarCreateProjectRequest {
    pub name: String,
    #[serde(default)]
    pub street: Option<String>,
    #[serde(default)]
    pub zipcode: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date: Option<String>,
}

/// Per-aspect toggles for copying a source project (`POST .../copy_project`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarCopyProjectOptions {
    pub name: String,
    #[serde(default)]
    pub details: bool,
    #[serde(default)]
    pub groups: bool,
    /// Forms in the Planradar UI.
    #[serde(default)]
    pub ticket_types: bool,
    #[serde(default)]
    pub users: bool,
    /// Layers in the Planradar UI.
    #[serde(default)]
    pub components: bool,
}

fn projects_path(customer_id: &str) -> String {
    format!("/api/v1/{customer_id}/projects")
}

fn project_path(customer_id: &str, project_id: &str) -> String {
    format!("/api/v1/{customer_id}/projects/{project_id}")
}

#[tauri::command]
#[specta::specta]
pub async fn planradar_get_project_status(
    app: tauri::AppHandle,
    project_id: String,
) -> Result<PlanradarProject, PlanradarApiError> {
    let (client, token, config) = build_client(app)?;
    read_project_status_core(&client, &token, &config.customer_id, &project_id).await
}

#[tauri::command]
#[specta::specta]
pub async fn planradar_list_projects(
    app: tauri::AppHandle,
    input: PlanradarListProjectsInput,
) -> Result<Vec<PlanradarProject>, PlanradarApiError> {
    let (client, token, config) = build_client(app)?;
    list_projects_core(&client, &token, &config.customer_id, &input).await
}

#[tauri::command]
#[specta::specta]
pub async fn planradar_create_project(
    app: tauri::AppHandle,
    request: PlanradarCreateProjectRequest,
) -> Result<String, PlanradarApiError> {
    let (client, token, config) = build_client(app)?;
    create_project_core(&client, &token, &config.customer_id, &request).await
}

#[tauri::command]
#[specta::specta]
pub async fn planradar_copy_project(
    app: tauri::AppHandle,
    project_id: String,
    options: PlanradarCopyProjectOptions,
) -> Result<String, PlanradarApiError> {
    let (client, token, config) = build_client(app)?;
    copy_project_core(&client, &token, &config.customer_id, &project_id, &options).await
}

#[tauri::command]
#[specta::specta]
pub async fn planradar_reactivate_project(
    app: tauri::AppHandle,
    project_id: String,
) -> Result<(), PlanradarApiError> {
    let (client, token, config) = build_client(app)?;
    reactivate_project_core(&client, &token, &config.customer_id, &project_id).await
}

fn build_client(
    app: tauri::AppHandle,
) -> Result<(PlanradarApiClient, String, PlanradarConfig), PlanradarApiError> {
    let store = load_store_or_error(app)?;
    let config = load_config(&store)?;
    let token = load_api_token()?;
    let client = PlanradarApiClient::new(&config.base_url)?;
    Ok((client, token, config))
}

pub(super) async fn read_project_status_core(
    client: &PlanradarApiClient,
    api_key: &str,
    customer_id: &str,
    project_id: &str,
) -> Result<PlanradarProject, PlanradarApiError> {
    let path = project_path(customer_id, project_id);
    let response = client
        .send_request(
            PlanradarHttpMethod::Get,
            &path,
            vec![],
            None,
            Some(api_key.to_string()),
        )
        .await?;

    let value = parse_success_json_body::<Value>(response.status, &response.body, &path)?;
    let data = value
        .get("data")
        .ok_or_else(|| missing_field_error(&path, "data"))?;
    project_from_data(data, &path)
}

pub(super) async fn list_projects_core(
    client: &PlanradarApiClient,
    api_key: &str,
    customer_id: &str,
    input: &PlanradarListProjectsInput,
) -> Result<Vec<PlanradarProject>, PlanradarApiError> {
    let path = projects_path(customer_id);
    let mut query = Vec::new();
    if let Some(sort) = &input.sort {
        query.push(("sort".to_string(), sort.clone()));
    }
    if let Some(page) = input.page {
        query.push(("page".to_string(), page.to_string()));
    }
    if let Some(pagesize) = input.pagesize {
        query.push(("pagesize".to_string(), pagesize.to_string()));
    }

    let response = client
        .send_request(
            PlanradarHttpMethod::Get,
            &path,
            query,
            None,
            Some(api_key.to_string()),
        )
        .await?;

    let value = parse_success_json_body::<Value>(response.status, &response.body, &path)?;
    let items = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| missing_field_error(&path, "data array"))?;

    items
        .iter()
        .map(|data| project_from_data(data, &path))
        .collect()
}

pub(super) async fn create_project_core(
    client: &PlanradarApiClient,
    api_key: &str,
    customer_id: &str,
    request: &PlanradarCreateProjectRequest,
) -> Result<String, PlanradarApiError> {
    let path = projects_path(customer_id);
    let body = build_create_project_body(request);

    let response = client
        .send_request(
            PlanradarHttpMethod::Post,
            &path,
            vec![],
            Some(body),
            Some(api_key.to_string()),
        )
        .await?;

    let value = parse_success_json_body::<Value>(response.status, &response.body, &path)?;
    extract_new_project_id(&value, &path)
}

pub(super) async fn copy_project_core(
    client: &PlanradarApiClient,
    api_key: &str,
    customer_id: &str,
    project_id: &str,
    options: &PlanradarCopyProjectOptions,
) -> Result<String, PlanradarApiError> {
    let path = format!("{}/copy_project", project_path(customer_id, project_id));
    let query = vec![
        ("name".to_string(), options.name.clone()),
        ("details".to_string(), options.details.to_string()),
        ("groups".to_string(), options.groups.to_string()),
        ("ticket_types".to_string(), options.ticket_types.to_string()),
        ("users".to_string(), options.users.to_string()),
        ("components".to_string(), options.components.to_string()),
    ];

    let response = client
        .send_request(
            PlanradarHttpMethod::Post,
            &path,
            query,
            None,
            Some(api_key.to_string()),
        )
        .await?;

    let value = parse_success_json_body::<Value>(response.status, &response.body, &path)?;
    extract_new_project_id(&value, &path)
}

pub(super) async fn reactivate_project_core(
    client: &PlanradarApiClient,
    api_key: &str,
    customer_id: &str,
    project_id: &str,
) -> Result<(), PlanradarApiError> {
    let path = format!("{}/archive_project", project_path(customer_id, project_id));
    let body = json!({
        "data": { "attributes": { "status": PLANRADAR_STATUS_ACTIVE } }
    });

    let response = client
        .send_request(
            PlanradarHttpMethod::Put,
            &path,
            vec![],
            Some(body),
            Some(api_key.to_string()),
        )
        .await?;

    if !(200..300).contains(&response.status) {
        return Err(super::shared::normalize_http_error(
            response.status,
            &response.body,
            &path,
        ));
    }

    Ok(())
}

fn build_create_project_body(request: &PlanradarCreateProjectRequest) -> Value {
    let mut attributes = Map::new();
    attributes.insert("name".to_string(), Value::String(request.name.clone()));
    insert_optional(&mut attributes, "street", &request.street);
    insert_optional(&mut attributes, "zipcode", &request.zipcode);
    insert_optional(&mut attributes, "city", &request.city);
    insert_optional(&mut attributes, "country", &request.country);
    insert_optional(&mut attributes, "description", &request.description);
    // The Planradar Open API documents the project start/end dates under the hyphenated
    // attribute keys `drstart-date` / `drend-date` (the variants that carry descriptions and
    // examples in the spec). Unknown attribute keys are silently ignored by the API, so these
    // must match exactly or the dates are dropped.
    insert_optional(&mut attributes, "drstart-date", &request.start_date);
    insert_optional(&mut attributes, "drend-date", &request.end_date);

    json!({ "data": { "attributes": Value::Object(attributes) } })
}

fn insert_optional(map: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_string(), Value::String(value.clone()));
    }
}

fn project_from_data(data: &Value, path: &str) -> Result<PlanradarProject, PlanradarApiError> {
    let id = value_to_id(data.get("id")).ok_or_else(|| missing_field_error(path, "data.id"))?;
    let attributes = data.get("attributes");
    let name = attributes
        .and_then(|attributes| attributes.get("name"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string();
    let status_code = attributes
        .and_then(|attributes| attributes.get("status"))
        .and_then(value_to_status_code)
        .unwrap_or(PLANRADAR_STATUS_ACTIVE);

    Ok(PlanradarProject {
        id,
        name,
        status: PlanradarProjectStatus::from_api_status(status_code),
    })
}

fn extract_new_project_id(value: &Value, path: &str) -> Result<String, PlanradarApiError> {
    value
        .get("data")
        .and_then(|data| value_to_id(data.get("id")))
        .ok_or_else(|| missing_field_error(path, "data.id"))
}

/// JSON:API ids are strings by spec, but Planradar may serialize them as integers; accept both.
fn value_to_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// The status attribute is normally an integer, but tolerate a numeric string too.
fn value_to_status_code(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn missing_field_error(path: &str, field: &str) -> PlanradarApiError {
    PlanradarApiError::new(
        PlanradarApiErrorCode::InvalidResponse,
        None,
        "Die Antwort von Planradar konnte nicht verarbeitet werden.",
        format!("Planradar-Antwort für {path} enthält kein Feld `{field}`."),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::planradar::client::{
        BoxFuture, PlanradarApiClient, PlanradarHttpMethod, PlanradarHttpRequest,
        PlanradarHttpResponse, PlanradarHttpTransport,
    };
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[derive(Clone)]
    struct MockTransport {
        responses: Arc<Mutex<VecDeque<Result<PlanradarHttpResponse, PlanradarApiError>>>>,
        requests: Arc<Mutex<Vec<PlanradarHttpRequest>>>,
    }

    impl MockTransport {
        fn new(responses: Vec<Result<PlanradarHttpResponse, PlanradarApiError>>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(VecDeque::from(responses))),
                requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn requests(&self) -> Vec<PlanradarHttpRequest> {
            self.requests
                .lock()
                .expect("request lock should succeed")
                .clone()
        }
    }

    impl PlanradarHttpTransport for MockTransport {
        fn send<'a>(
            &'a self,
            request: PlanradarHttpRequest,
        ) -> BoxFuture<'a, Result<PlanradarHttpResponse, PlanradarApiError>> {
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

    fn mock_response(status: u16, body: &str) -> PlanradarHttpResponse {
        PlanradarHttpResponse {
            status,
            body: body.to_string(),
        }
    }

    #[test]
    fn request_attaches_api_key_header_and_builds_customer_path() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":{"id":"42","attributes":{"name":"Projekt","status":1}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            read_project_status_core(&client, "secret-token", "1234", "42")
                .await
                .expect("status read should succeed");

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/api/v1/1234/projects/42");
            assert_eq!(requests[0].method, PlanradarHttpMethod::Get);
            assert_eq!(requests[0].api_key, Some("secret-token".to_string()));
        });
    }

    #[test]
    fn read_project_status_maps_active_and_archived() {
        tauri::async_runtime::block_on(async {
            let active = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":{"id":"1","attributes":{"name":" Aktiv ","status":1}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(active));
            let project = read_project_status_core(&client, "t", "1234", "1")
                .await
                .expect("active status should parse");
            assert_eq!(project.id, "1");
            assert_eq!(project.name, "Aktiv");
            assert_eq!(project.status, PlanradarProjectStatus::Active);

            let archived = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":{"id":"2","attributes":{"name":"Archiviert","status":9}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(archived));
            let project = read_project_status_core(&client, "t", "1234", "2")
                .await
                .expect("archived status should parse");
            assert_eq!(project.status, PlanradarProjectStatus::Archived);
        });
    }

    #[test]
    fn list_projects_sends_pagination_query_and_maps_results() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":[
                    {"id":"1","attributes":{"name":"A","status":1}},
                    {"id":2,"attributes":{"name":"B","status":9}}
                ]}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            let projects = list_projects_core(
                &client,
                "t",
                "1234",
                &PlanradarListProjectsInput {
                    sort: Some("name".to_string()),
                    page: Some(2),
                    pagesize: Some(50),
                },
            )
            .await
            .expect("list should succeed");

            assert_eq!(projects.len(), 2);
            assert_eq!(projects[0].name, "A");
            assert_eq!(projects[1].id, "2");
            assert_eq!(projects[1].status, PlanradarProjectStatus::Archived);

            let requests = transport.requests();
            assert_eq!(requests[0].path, "/api/v1/1234/projects");
            assert_eq!(
                requests[0].query,
                vec![
                    ("sort".to_string(), "name".to_string()),
                    ("page".to_string(), "2".to_string()),
                    ("pagesize".to_string(), "50".to_string()),
                ]
            );
        });
    }

    #[test]
    fn create_project_sends_attributes_body_and_returns_new_id() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                201,
                r#"{"data":{"id":"9001","attributes":{"name":"Neu","status":1}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            let new_id = create_project_core(
                &client,
                "t",
                "1234",
                &PlanradarCreateProjectRequest {
                    name: "Neu".to_string(),
                    city: Some("Wien".to_string()),
                    start_date: Some("2026-02-23T10:02:25.000Z".to_string()),
                    ..PlanradarCreateProjectRequest::default()
                },
            )
            .await
            .expect("create should succeed");

            assert_eq!(new_id, "9001");

            let requests = transport.requests();
            assert_eq!(requests[0].method, PlanradarHttpMethod::Post);
            assert_eq!(requests[0].path, "/api/v1/1234/projects");
            let body = requests[0]
                .body
                .as_ref()
                .expect("create should send a body");
            let attributes = &body["data"]["attributes"];
            assert_eq!(attributes["name"], "Neu");
            assert_eq!(attributes["city"], "Wien");
            assert_eq!(attributes["drstart-date"], "2026-02-23T10:02:25.000Z");
            // Unset optional fields must be omitted entirely.
            assert!(attributes.get("street").is_none());
            assert!(attributes.get("drend-date").is_none());
        });
    }

    #[test]
    fn copy_project_maps_name_and_toggles_to_query_params() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":{"id":"7777","attributes":{"name":"Kopie","status":1}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            let new_id = copy_project_core(
                &client,
                "t",
                "1234",
                "42",
                &PlanradarCopyProjectOptions {
                    name: "Kopie".to_string(),
                    details: true,
                    groups: false,
                    ticket_types: true,
                    users: false,
                    components: true,
                },
            )
            .await
            .expect("copy should succeed");

            assert_eq!(new_id, "7777");

            let requests = transport.requests();
            assert_eq!(requests[0].method, PlanradarHttpMethod::Post);
            assert_eq!(requests[0].path, "/api/v1/1234/projects/42/copy_project");
            assert_eq!(
                requests[0].query,
                vec![
                    ("name".to_string(), "Kopie".to_string()),
                    ("details".to_string(), "true".to_string()),
                    ("groups".to_string(), "false".to_string()),
                    ("ticket_types".to_string(), "true".to_string()),
                    ("users".to_string(), "false".to_string()),
                    ("components".to_string(), "true".to_string()),
                ]
            );
        });
    }

    #[test]
    fn reactivate_sends_archive_project_with_status_one() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(200, r#"{"data":{}}"#))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            reactivate_project_core(&client, "t", "1234", "42")
                .await
                .expect("reactivate should succeed");

            let requests = transport.requests();
            assert_eq!(requests[0].method, PlanradarHttpMethod::Put);
            assert_eq!(requests[0].path, "/api/v1/1234/projects/42/archive_project");
            let body = requests[0]
                .body
                .as_ref()
                .expect("reactivate should send a body");
            assert_eq!(body["data"]["attributes"]["status"], 1);
        });
    }

    #[test]
    fn maps_auth_failure_to_unauthorized_error() {
        tauri::async_runtime::block_on(async {
            let transport =
                MockTransport::new(vec![Ok(mock_response(401, r#"{"error":"invalid key"}"#))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport));

            let error = read_project_status_core(&client, "t", "1234", "1")
                .await
                .expect_err("401 should map to an error");
            assert_eq!(error.code, PlanradarApiErrorCode::Unauthorized);
            assert_eq!(error.http_status, Some(401));
            assert!(error.user_message.contains("Planradar"));
        });
    }

    #[test]
    fn maps_not_found_to_not_found_error() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(404, "Project Not Found"))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport));

            let error = read_project_status_core(&client, "t", "1234", "999")
                .await
                .expect_err("404 should map to an error");
            assert_eq!(error.code, PlanradarApiErrorCode::NotFound);
        });
    }

    #[test]
    fn maps_rate_limit_to_rate_limited_error() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(429, "slow down"))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport));

            let error = read_project_status_core(&client, "t", "1234", "1")
                .await
                .expect_err("429 should map to an error");
            assert_eq!(error.code, PlanradarApiErrorCode::RateLimited);
        });
    }

    #[test]
    fn malformed_response_maps_to_invalid_response() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(200, "not json {{{"))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport));

            let error = read_project_status_core(&client, "t", "1234", "1")
                .await
                .expect_err("malformed body should fail");
            assert_eq!(error.code, PlanradarApiErrorCode::InvalidResponse);
        });
    }

    #[test]
    #[ignore = "requires a recorded git-crypt cassette: record via recording_harness with live Planradar creds and VCR_MODE=record"]
    fn read_project_status_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = PlanradarApiClient::with_replay_cassette("planradar-get-project.json")
                .expect("replay client should be created");

            let project = read_project_status_core(&client, "replay-token", "1234", "1")
                .await
                .expect("status read should replay from cassette");

            assert_eq!(project.id, "1");
            assert!(!project.name.is_empty());
        });
    }

    #[test]
    #[ignore = "requires a recorded git-crypt cassette: record via recording_harness with live Planradar creds and VCR_MODE=record"]
    fn list_projects_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = PlanradarApiClient::with_replay_cassette("planradar-list-projects.json")
                .expect("replay client should be created");

            let projects = list_projects_core(
                &client,
                "replay-token",
                "1234",
                &PlanradarListProjectsInput {
                    sort: Some("name".to_string()),
                    page: Some(1),
                    pagesize: Some(10),
                },
            )
            .await
            .expect("list should replay from cassette");

            assert!(!projects.is_empty());
            assert!(projects.iter().all(|project| !project.id.is_empty()));
        });
    }

    #[test]
    #[ignore = "requires a recorded git-crypt cassette: record via recording_harness with live Planradar creds and VCR_MODE=record"]
    fn create_project_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = PlanradarApiClient::with_replay_cassette("planradar-create-project.json")
                .expect("replay client should be created");

            let new_id = create_project_core(
                &client,
                "replay-token",
                "1234",
                &PlanradarCreateProjectRequest {
                    name: "Cassette Projekt".to_string(),
                    ..PlanradarCreateProjectRequest::default()
                },
            )
            .await
            .expect("create should replay from cassette");

            assert!(!new_id.is_empty());
        });
    }

    #[test]
    #[ignore = "requires a recorded git-crypt cassette: record via recording_harness with live Planradar creds and VCR_MODE=record"]
    fn copy_project_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = PlanradarApiClient::with_replay_cassette("planradar-copy-project.json")
                .expect("replay client should be created");

            let new_id = copy_project_core(
                &client,
                "replay-token",
                "1234",
                "1",
                &PlanradarCopyProjectOptions {
                    name: "Cassette Projekt (Kopie)".to_string(),
                    details: true,
                    groups: true,
                    ticket_types: true,
                    users: false,
                    components: true,
                },
            )
            .await
            .expect("copy should replay from cassette");

            assert!(!new_id.is_empty());
        });
    }

    #[test]
    #[ignore = "requires a recorded git-crypt cassette: record via recording_harness with live Planradar creds and VCR_MODE=record"]
    fn reactivate_project_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client =
                PlanradarApiClient::with_replay_cassette("planradar-reactivate-project.json")
                    .expect("replay client should be created");

            reactivate_project_core(&client, "replay-token", "1234", "1")
                .await
                .expect("reactivate should replay from cassette");
        });
    }
}
