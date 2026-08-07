use super::client::{PlanradarApiClient, PlanradarHttpMethod};
use super::shared::{
    load_api_token, load_config, load_store_or_error, parse_success_json_body, truncate_for_log,
    validate_path_segment, PlanradarApiError, PlanradarApiErrorCode, PlanradarConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlanradarProjectStatus {
    Active,
    Archived,
}

const PLANRADAR_STATUS_ACTIVE: i64 = 1;
const PLANRADAR_STATUS_ARCHIVED: i64 = 9;

impl PlanradarProjectStatus {
    /// An unrecognized code is rejected rather than defaulted: a wrong "active" would silently
    /// skip a needed reactivation.
    fn from_api_status(status: i64) -> Option<Self> {
        match status {
            PLANRADAR_STATUS_ACTIVE => Some(Self::Active),
            PLANRADAR_STATUS_ARCHIVED => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlanradarProject {
    pub id: String,
    pub name: String,
    pub status: PlanradarProjectStatus,
}

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

fn project_path(customer_id: &str, project_id: &str) -> Result<String, PlanradarApiError> {
    let project_id = validate_path_segment(
        project_id,
        "projectId",
        "Die Planradar-Projekt-ID ist ungültig.",
    )?;

    Ok(format!("/api/v1/{customer_id}/projects/{project_id}"))
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

/// Returns a job ID, not a project ID: Planradar copies asynchronously, and the copied project
/// only exists once that job finishes.
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
    let path = project_path(customer_id, project_id)?;
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
        .ok_or_else(|| missing_field_error(&path, "data", &value))?;
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
        .ok_or_else(|| missing_field_error(&path, "data array", &value))?;

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
    let path = format!("{}/copy_project", project_path(customer_id, project_id)?);
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
    extract_job_id(&value, &path)
}

pub(super) async fn reactivate_project_core(
    client: &PlanradarApiClient,
    api_key: &str,
    customer_id: &str,
    project_id: &str,
) -> Result<(), PlanradarApiError> {
    let path = format!("{}/archive_project", project_path(customer_id, project_id)?);
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
    // Planradar silently ignores unknown attribute keys, so these hyphenated date keys must
    // match the spec exactly or the dates are dropped.
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
    let id =
        value_to_id(data.get("id")).ok_or_else(|| missing_field_error(path, "data.id", data))?;
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
        .ok_or_else(|| missing_field_error(path, "data.attributes.status", data))?;
    let status = PlanradarProjectStatus::from_api_status(status_code).ok_or_else(|| {
        PlanradarApiError::new(
            PlanradarApiErrorCode::InvalidResponse,
            None,
            "Die Antwort von Planradar konnte nicht verarbeitet werden.",
            format!("Planradar-Antwort für {path} enthält unbekannten Status {status_code}."),
        )
    })?;

    Ok(PlanradarProject { id, name, status })
}

fn extract_new_project_id(value: &Value, path: &str) -> Result<String, PlanradarApiError> {
    value
        .get("data")
        .and_then(|data| value_to_id(data.get("id")))
        .ok_or_else(|| missing_field_error(path, "data.id", value))
}

fn extract_job_id(value: &Value, path: &str) -> Result<String, PlanradarApiError> {
    value_to_id(value.get("job_id")).ok_or_else(|| missing_field_error(path, "job_id", value))
}

/// Request shapes the recording harness sends when cutting new cassettes. Replay tests do not
/// use these; they rebuild their inputs from the cassette they replay.
#[cfg(test)]
pub(super) mod vcr_fixtures {
    use super::{
        PlanradarCopyProjectOptions, PlanradarCreateProjectRequest, PlanradarListProjectsInput,
    };

    fn env_or(key: &str, fallback: &str) -> String {
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| fallback.to_string())
    }

    pub(in crate::integrations::planradar) fn new_project_name() -> String {
        env_or("PLANRADAR_VCR_NEW_PROJECT_NAME", "Cassette Projekt")
    }

    pub(in crate::integrations::planradar) fn list_input() -> PlanradarListProjectsInput {
        PlanradarListProjectsInput {
            sort: Some("name".to_string()),
            page: Some(1),
            pagesize: Some(10),
        }
    }

    pub(in crate::integrations::planradar) fn create_request() -> PlanradarCreateProjectRequest {
        PlanradarCreateProjectRequest {
            name: new_project_name(),
            // Dates are sent so a recorded cassette proves Planradar accepts the drstart-date /
            // drend-date keys instead of dropping them.
            city: Some("Wien".to_string()),
            country: Some("Österreich".to_string()),
            start_date: Some("2026-02-23T10:02:25.000Z".to_string()),
            end_date: Some("2026-02-26T00:00:00.000Z".to_string()),
            ..PlanradarCreateProjectRequest::default()
        }
    }

    pub(in crate::integrations::planradar) fn copy_options() -> PlanradarCopyProjectOptions {
        PlanradarCopyProjectOptions {
            name: format!("{} (Kopie)", new_project_name()),
            details: true,
            groups: true,
            ticket_types: true,
            users: false,
            components: true,
        }
    }
}

/// JSON:API ids are strings by spec, but Planradar may serialize them as integers; accept both.
fn value_to_id(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// Planradar sometimes serializes the status attribute as a numeric string.
fn value_to_status_code(value: &Value) -> Option<i64> {
    match value {
        Value::Number(n) => n.as_i64(),
        Value::String(s) => s.trim().parse::<i64>().ok(),
        _ => None,
    }
}

/// Includes the received payload: several Planradar success responses are undocumented, so the
/// actual shape is the only way to diagnose a mismatch without a live recording.
fn missing_field_error(path: &str, field: &str, received: &Value) -> PlanradarApiError {
    PlanradarApiError::new(
        PlanradarApiErrorCode::InvalidResponse,
        None,
        "Die Antwort von Planradar konnte nicht verarbeitet werden.",
        format!(
            "Planradar-Antwort für {path} enthält kein Feld `{field}`; empfangen: {}",
            truncate_for_log(&received.to_string())
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::http_record_replay::RecordedRequest;
    use crate::integrations::planradar::client::{
        cassette_path_for_test, BoxFuture, PlanradarApiClient, PlanradarHttpMethod,
        PlanradarHttpRequest, PlanradarHttpResponse, PlanradarHttpTransport,
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
            assert!(attributes.get("street").is_none());
            assert!(attributes.get("drend-date").is_none());
        });
    }

    #[test]
    fn copy_project_maps_name_and_toggles_to_query_params() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                202,
                r#"{"job_id":"f2f8a66e-39bb-4baa-a38c-0f09e4758e07"}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            let job_id = copy_project_core(
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

            assert_eq!(job_id, "f2f8a66e-39bb-4baa-a38c-0f09e4758e07");

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
    fn missing_status_is_rejected_instead_of_defaulting_to_active() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":{"id":"1","attributes":{"name":"Ohne Status"}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport));

            let error = read_project_status_core(&client, "t", "1234", "1")
                .await
                .expect_err("missing status should not silently default to Active");
            assert_eq!(error.code, PlanradarApiErrorCode::InvalidResponse);
        });
    }

    #[test]
    fn unknown_status_code_is_rejected() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"data":{"id":"1","attributes":{"name":"Seltsam","status":4}}}"#,
            ))]);
            let client = PlanradarApiClient::with_transport(Box::new(transport));

            let error = read_project_status_core(&client, "t", "1234", "1")
                .await
                .expect_err("unknown status code should be rejected");
            assert_eq!(error.code, PlanradarApiErrorCode::InvalidResponse);
            assert!(error.technical_message.contains('4'));
        });
    }

    #[test]
    fn rejects_project_id_that_escapes_the_customer_scope() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![]);
            let client = PlanradarApiClient::with_transport(Box::new(transport.clone()));

            let error = read_project_status_core(&client, "t", "1234", "../../9999/projects/5")
                .await
                .expect_err("path-traversing project id should be rejected");
            assert_eq!(error.code, PlanradarApiErrorCode::InvalidConfiguration);
            assert!(
                transport.requests().is_empty(),
                "no request should be sent for an invalid project id"
            );
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
    fn read_project_status_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let recorded = recorded_request("planradar-get-project.json");
            let client = PlanradarApiClient::with_replay_cassette("planradar-get-project.json")
                .expect("replay client should be created");

            let project = read_project_status_core(
                &client,
                "replay-token",
                &recorded_customer_id(&recorded),
                &recorded_project_id(&recorded),
            )
            .await
            .expect("status read should replay from cassette");

            assert!(!project.id.is_empty());
            assert!(!project.name.is_empty());
        });
    }

    #[test]
    fn list_projects_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let recorded = recorded_request("planradar-list-projects.json");
            let client = PlanradarApiClient::with_replay_cassette("planradar-list-projects.json")
                .expect("replay client should be created");

            let projects = list_projects_core(
                &client,
                "replay-token",
                &recorded_customer_id(&recorded),
                &PlanradarListProjectsInput {
                    sort: recorded_query(&recorded, "sort"),
                    page: recorded_query(&recorded, "page")
                        .map(|page| page.parse().expect("recorded page should be numeric")),
                    pagesize: recorded_query(&recorded, "pagesize").map(|pagesize| {
                        pagesize
                            .parse()
                            .expect("recorded pagesize should be numeric")
                    }),
                },
            )
            .await
            .expect("list should replay from cassette");

            assert!(!projects.is_empty());
            assert!(projects.iter().all(|project| !project.id.is_empty()));
        });
    }

    #[test]
    fn create_project_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let recorded = recorded_request("planradar-create-project.json");
            let attributes = recorded.body.as_ref().expect("create should record a body")["data"]
                ["attributes"]
                .clone();
            let attribute = |key: &str| {
                attributes
                    .get(key)
                    .and_then(Value::as_str)
                    .map(str::to_string)
            };
            let client = PlanradarApiClient::with_replay_cassette("planradar-create-project.json")
                .expect("replay client should be created");

            let new_id = create_project_core(
                &client,
                "replay-token",
                &recorded_customer_id(&recorded),
                &PlanradarCreateProjectRequest {
                    name: attribute("name").expect("create should record a name"),
                    street: attribute("street"),
                    zipcode: attribute("zipcode"),
                    city: attribute("city"),
                    country: attribute("country"),
                    description: attribute("description"),
                    start_date: attribute("drstart-date"),
                    end_date: attribute("drend-date"),
                },
            )
            .await
            .expect("create should replay from cassette");

            assert!(!new_id.is_empty());
        });
    }

    #[test]
    fn copy_project_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let recorded = recorded_request("planradar-copy-project.json");
            let toggle = |key: &str| recorded_query(&recorded, key).as_deref() == Some("true");
            let client = PlanradarApiClient::with_replay_cassette("planradar-copy-project.json")
                .expect("replay client should be created");

            let job_id = copy_project_core(
                &client,
                "replay-token",
                &recorded_customer_id(&recorded),
                &recorded_project_id(&recorded),
                &PlanradarCopyProjectOptions {
                    name: recorded_query(&recorded, "name").expect("copy should record a name"),
                    details: toggle("details"),
                    groups: toggle("groups"),
                    ticket_types: toggle("ticket_types"),
                    users: toggle("users"),
                    components: toggle("components"),
                },
            )
            .await
            .expect("copy should replay from cassette");

            assert!(!job_id.is_empty());
        });
    }

    #[test]
    #[ignore = "no planradar-reactivate-project.json cassette has been recorded yet"]
    fn reactivate_project_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let recorded = recorded_request("planradar-reactivate-project.json");
            let client =
                PlanradarApiClient::with_replay_cassette("planradar-reactivate-project.json")
                    .expect("replay client should be created");

            reactivate_project_core(
                &client,
                "replay-token",
                &recorded_customer_id(&recorded),
                &recorded_project_id(&recorded),
            )
            .await
            .expect("reactivate should replay from cassette");
        });
    }

    /// Replay inputs are rebuilt from the cassette rather than from `PLANRADAR_*` env vars, so
    /// the tests pass without the account the cassettes were recorded against.
    fn recorded_request(cassette_file_name: &str) -> RecordedRequest {
        let path = cassette_path_for_test(cassette_file_name);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("cassette {} should be readable: {error}", path.display())
        });
        let cassette: Value =
            serde_json::from_str(&content).expect("cassette should contain valid JSON");

        serde_json::from_value(cassette["interactions"][0]["request"].clone())
            .expect("cassette should contain a recorded request")
    }

    fn path_segments(request: &RecordedRequest) -> Vec<&str> {
        request.path.trim_matches('/').split('/').collect()
    }

    fn recorded_customer_id(request: &RecordedRequest) -> String {
        path_segments(request)[2].to_string()
    }

    fn recorded_project_id(request: &RecordedRequest) -> String {
        path_segments(request)[4].to_string()
    }

    fn recorded_query(request: &RecordedRequest, key: &str) -> Option<String> {
        request
            .query
            .iter()
            .find(|(recorded_key, _)| recorded_key == key)
            .map(|(_, value)| value.clone())
    }
}
