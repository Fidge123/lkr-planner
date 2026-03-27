use super::auth_flow::send_authenticated_json;
use super::client::DayliteApiClient;
use super::client::DayliteHttpMethod;
use super::shared::{
    build_limit_query, load_daylite_tokens, load_store_or_error, save_store_or_error,
    store_daylite_tokens, DayliteApiError, DayliteSearchInput, DayliteSearchResult,
    DayliteTokenState,
};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteProjectSummary {
    #[serde(rename = "self")]
    pub reference: String,
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub started: Option<String>,
    #[serde(default)]
    pub completed: Option<String>,
    #[serde(default, alias = "create_date")]
    pub create_date: Option<String>,
    #[serde(default, alias = "modify_date")]
    pub modify_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanningProjectStatus {
    NewStatus,
    InProgress,
    Done,
    Abandoned,
    Cancelled,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct PlanningProjectRecord {
    #[serde(rename = "self")]
    pub reference: String,
    pub name: String,
    pub status: PlanningProjectStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub create_date: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modify_date: Option<String>,
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_projects(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningProjectRecord>, DayliteApiError> {
    let store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let (projects, token_state) = list_projects_core(&client, load_daylite_tokens()?).await?;

    store_daylite_tokens(&token_state)?;
    save_store_or_error(app, store)?;

    Ok(projects)
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_search_projects(
    app: tauri::AppHandle,
    input: DayliteSearchInput,
) -> Result<DayliteSearchResult<DayliteProjectSummary>, DayliteApiError> {
    let store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let (search_result, token_state) =
        search_projects_core(&client, load_daylite_tokens()?, &input).await?;

    store_daylite_tokens(&token_state)?;
    save_store_or_error(app, store)?;

    Ok(search_result)
}

pub(super) async fn list_projects_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
) -> Result<(Vec<PlanningProjectRecord>, DayliteTokenState), DayliteApiError> {
    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummary>>(
            client,
            token_state,
            DayliteHttpMethod::Post,
            "/projects/_search",
            vec![("full-records".to_string(), "true".to_string())],
            Some(json!({})),
        )
        .await?;

    let projects = search_result
        .results
        .into_iter()
        .map(map_daylite_project_summary)
        .collect();

    Ok((projects, token_state))
}

pub(super) async fn search_projects_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    input: &DayliteSearchInput,
) -> Result<
    (
        DayliteSearchResult<DayliteProjectSummary>,
        DayliteTokenState,
    ),
    DayliteApiError,
> {
    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummary>>(
            client,
            token_state,
            DayliteHttpMethod::Post,
            "/projects/_search",
            build_limit_query(input.limit),
            Some(json!({
                "name": {
                    "contains": input.search_term
                }
            })),
        )
        .await?;

    Ok((
        DayliteSearchResult {
            results: search_result
                .results
                .into_iter()
                .map(normalize_project_summary)
                .collect(),
            next: normalize_optional_string(search_result.next),
        },
        token_state,
    ))
}

fn map_daylite_project_summary(project: DayliteProjectSummary) -> PlanningProjectRecord {
    let project = normalize_project_summary(project);

    PlanningProjectRecord {
        reference: project.reference,
        name: project.name,
        status: map_project_status(project.status),
        category: project.category,
        keywords: project.keywords,
        due: project.due,
        started: project.started,
        completed: project.completed,
        create_date: project.create_date,
        modify_date: project.modify_date,
    }
}

fn normalize_project_summary(project: DayliteProjectSummary) -> DayliteProjectSummary {
    DayliteProjectSummary {
        reference: normalize_reference(project.reference),
        name: normalize_required_string(project.name),
        status: normalize_optional_string(project.status),
        category: normalize_optional_string(project.category),
        keywords: normalize_keywords(project.keywords),
        due: normalize_optional_date(project.due),
        started: normalize_optional_date(project.started),
        completed: normalize_optional_date(project.completed),
        create_date: normalize_optional_date(project.create_date),
        modify_date: normalize_optional_date(project.modify_date),
    }
}

fn normalize_required_string(value: String) -> String {
    value.trim().to_string()
}

fn normalize_reference(value: String) -> String {
    normalize_required_string(value)
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|candidate| {
        let normalized = candidate.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

fn normalize_keywords(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .filter_map(|value| {
            let normalized = value.trim();
            if normalized.is_empty() {
                None
            } else {
                Some(normalized.to_string())
            }
        })
        .collect()
}

fn normalize_optional_date(value: Option<String>) -> Option<String> {
    let raw_value = normalize_optional_string(value)?;

    if let Ok(parsed_date_time) = DateTime::parse_from_rfc3339(&raw_value) {
        return Some(
            parsed_date_time
                .with_timezone(&Utc)
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        );
    }

    if let Ok(parsed_date) = NaiveDate::parse_from_str(&raw_value, "%Y-%m-%d") {
        let start_of_day = parsed_date.and_hms_milli_opt(0, 0, 0, 0)?;
        let utc_date_time = DateTime::<Utc>::from_naive_utc_and_offset(start_of_day, Utc);
        return Some(utc_date_time.to_rfc3339_opts(SecondsFormat::Millis, true));
    }

    None
}

fn map_project_status(status: Option<String>) -> PlanningProjectStatus {
    let normalized = normalize_optional_string(status)
        .map(|value| value.to_lowercase())
        .unwrap_or_default();

    if normalized == "in_progress" {
        return PlanningProjectStatus::InProgress;
    }
    if normalized == "done" {
        return PlanningProjectStatus::Done;
    }
    if normalized == "abandoned" {
        return PlanningProjectStatus::Abandoned;
    }
    if normalized == "cancelled" {
        return PlanningProjectStatus::Cancelled;
    }
    if normalized == "deferred" {
        return PlanningProjectStatus::Deferred;
    }
    if normalized == "new" || normalized == "new_status" {
        return PlanningProjectStatus::NewStatus;
    }

    PlanningProjectStatus::NewStatus
}

#[cfg(test)]
mod tests {
    use super::{
        list_projects_core, map_daylite_project_summary, map_project_status, search_projects_core,
        DayliteProjectSummary, PlanningProjectStatus,
    };
    use crate::integrations::daylite::client::{
        BoxFuture, DayliteApiClient, DayliteHttpMethod, DayliteHttpRequest, DayliteHttpResponse,
        DayliteHttpTransport,
    };
    use crate::integrations::daylite::shared::{
        DayliteApiError, DayliteSearchInput, DayliteTokenState,
    };
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    #[test]
    fn maps_project_summary_to_planning_project_record() {
        let project = DayliteProjectSummary {
            reference: " /v1/projects/7000 ".to_string(),
            name: " Projekt Nord ".to_string(),
            status: Some(" NEW ".to_string()),
            category: Some(" Überfällig ".to_string()),
            keywords: vec![
                " Aufträge ".to_string(),
                "".to_string(),
                "Vorbereitung".to_string(),
            ],
            due: Some("2026-02-15".to_string()),
            started: None,
            completed: None,
            create_date: Some("not-a-date".to_string()),
            modify_date: Some("2026-02-15T12:45:00+01:00".to_string()),
        };

        let mapped = map_daylite_project_summary(project);

        assert_eq!(mapped.reference, "/v1/projects/7000");
        assert_eq!(mapped.name, "Projekt Nord");
        assert_eq!(mapped.status, PlanningProjectStatus::NewStatus);
        assert_eq!(mapped.category, Some("Überfällig".to_string()));
        assert_eq!(
            mapped.keywords,
            vec!["Aufträge".to_string(), "Vorbereitung".to_string()]
        );
        assert_eq!(mapped.due, Some("2026-02-15T00:00:00.000Z".to_string()));
        assert_eq!(mapped.create_date, None);
        assert_eq!(
            mapped.modify_date,
            Some("2026-02-15T11:45:00.000Z".to_string())
        );
    }

    #[test]
    fn defaults_unknown_project_status_to_new_status() {
        let mapped_status = map_project_status(Some("unknown-status".to_string()));
        assert_eq!(mapped_status, PlanningProjectStatus::NewStatus);
    }

    #[test]
    fn list_projects_sends_search_request_and_maps_results() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"results":[{"self":"/v1/projects/1","name":"Projekt A","status":"in_progress"},{"self":"/v1/projects/2","name":"Projekt B"}],"next":null}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let (projects, token_state) = list_projects_core(
                &client,
                DayliteTokenState {
                    access_token: "at".to_string(),
                    refresh_token: "rt".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
            )
            .await
            .expect("list should succeed");

            assert_eq!(projects.len(), 2);
            assert_eq!(projects[0].name, "Projekt A");
            assert_eq!(projects[0].status, PlanningProjectStatus::InProgress);
            assert_eq!(projects[1].name, "Projekt B");
            assert_eq!(projects[1].status, PlanningProjectStatus::NewStatus);
            assert_eq!(token_state.access_token, "at");

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(requests[0].path, "/projects/_search");
            assert_eq!(requests[0].method, DayliteHttpMethod::Post);
            assert_eq!(
                requests[0].query,
                vec![("full-records".to_string(), "true".to_string())]
            );
            assert!(requests[0].body.is_some());
        });
    }

    #[test]
    fn search_projects_sends_correct_body_and_query() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![Ok(mock_response(
                200,
                r#"{"results":[{"self":" /v1/projects/10 ","name":" Projekt Nord ","category":" Bau ","keywords":[" Aufträge ",""],"due":"2026-02-15"}],"next":" /v1/projects/_search?offset=5 "}"#,
            ))]);
            let client = DayliteApiClient::with_transport(Arc::new(transport.clone()));

            let (result, _) = search_projects_core(
                &client,
                DayliteTokenState {
                    access_token: "at".to_string(),
                    refresh_token: "rt".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
                &DayliteSearchInput {
                    search_term: "Nord".to_string(),
                    limit: Some(5),
                },
            )
            .await
            .expect("search should succeed");

            assert_eq!(result.results.len(), 1);
            assert_eq!(result.results[0].reference, "/v1/projects/10");
            assert_eq!(result.results[0].name, "Projekt Nord");
            assert_eq!(result.results[0].category, Some("Bau".to_string()));
            assert_eq!(result.results[0].keywords, vec!["Aufträge".to_string()]);
            assert_eq!(
                result.results[0].due,
                Some("2026-02-15T00:00:00.000Z".to_string())
            );
            assert_eq!(
                result.next,
                Some("/v1/projects/_search?offset=5".to_string())
            );

            let requests = transport.requests();
            assert_eq!(requests.len(), 1);
            assert_eq!(
                requests[0].query,
                vec![("limit".to_string(), "5".to_string())]
            );
            let body = requests[0].body.as_ref().expect("should have body");
            assert_eq!(body["name"]["contains"], "Nord");
        });
    }

    #[test]
    fn list_projects_returns_updated_token_state_after_refresh() {
        tauri::async_runtime::block_on(async {
            let transport = MockTransport::new(vec![
                Ok(mock_response(
                    200,
                    r#"{"access_token":"new-at","refresh_token":"new-rt","expires_in":3600}"#,
                )),
                Ok(mock_response(200, r#"{"results":[],"next":null}"#)),
            ]);
            let client = DayliteApiClient::with_transport(Arc::new(transport));

            let (projects, token_state) = list_projects_core(
                &client,
                DayliteTokenState {
                    access_token: String::new(),
                    refresh_token: "old-rt".to_string(),
                    access_token_expires_at_ms: None,
                },
            )
            .await
            .expect("list after refresh should succeed");

            assert!(projects.is_empty());
            assert_eq!(token_state.access_token, "new-at");
            assert_eq!(token_state.refresh_token, "new-rt");
            assert!(token_state.access_token_expires_at_ms.is_some());
        });
    }

    #[test]
    fn list_projects_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = DayliteApiClient::with_replay_cassette("daylite-list-projects.json")
                .expect("replay client should be created");

            let (projects, token_state) = list_projects_core(
                &client,
                DayliteTokenState {
                    access_token: "replay-access-token".to_string(),
                    refresh_token: "replay-refresh-token".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
            )
            .await
            .expect("list should replay from cassette");

            assert!(!projects.is_empty());
            assert!(
                projects
                    .iter()
                    .all(|project| project.reference.starts_with("/v1/projects/"))
            );
            assert!(
                projects
                    .iter()
                    .all(|project| !project.name.is_empty() && project.name == project.name.trim())
            );
            assert_eq!(token_state.access_token, "replay-access-token");
        });
    }

    #[test]
    fn search_projects_replays_vcr_cassette() {
        tauri::async_runtime::block_on(async {
            let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
                .expect("replay client should be created");

            let (search_result, token_state) = search_projects_core(
                &client,
                DayliteTokenState {
                    access_token: "replay-access-token".to_string(),
                    refresh_token: "replay-refresh-token".to_string(),
                    access_token_expires_at_ms: Some(u64::MAX),
                },
                &DayliteSearchInput {
                    search_term: "Nord".to_string(),
                    limit: Some(5),
                },
            )
            .await
            .expect("search should replay from cassette");

            assert!(!search_result.results.is_empty());
            assert!(search_result.results.len() <= 5);
            assert!(search_result.results.iter().all(|project| {
                project.reference.starts_with("/v1/projects/")
                    && !project.name.is_empty()
                    && project.name == project.name.trim()
                    && project.name.to_lowercase().contains("nord")
            }));
            assert!(
                search_result
                    .next
                    .as_deref()
                    .map(|next| next.starts_with("/v1/projects/_search"))
                    .unwrap_or(true)
            );
            assert_eq!(token_state.access_token, "replay-access-token");
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
