use super::auth_flow::send_authenticated_json;
use super::client::DayliteApiClient;
use super::client::DayliteHttpMethod;
use super::client::DayliteHttpRequest;
use super::project_cache::{cache_now_ms, project_cache};
use super::shared::{
    build_limit_query, run_daylite_command, trimmed, trimmed_or_none, with_daylite_tokens,
    DayliteApiError, DayliteSearchInput, DayliteSearchResult, DayliteSearchSort, DayliteTokenState,
};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
struct DayliteProjectSummaryDto {
    #[serde(rename = "self")]
    reference: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    keywords: Vec<String>,
    #[serde(default)]
    due: Option<String>,
    #[serde(default)]
    started: Option<String>,
    #[serde(default)]
    completed: Option<String>,
    #[serde(default)]
    create_date: Option<String>,
    #[serde(default)]
    modify_date: Option<String>,
}

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
    #[serde(default)]
    pub create_date: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub create_date: Option<String>,
    #[serde(default)]
    pub modify_date: Option<String>,
}

const OVERDUE_CATEGORY: &str = "Überfällig";
pub(crate) const FIXED_APPOINTMENT_CATEGORY: &str = "Termin FIX geplant";
// The Daylite API has no multi-value operator for scalar fields, so the overdue
// query pairs the category filter with each status as OR clauses to stay a
// single call.
const OVERDUE_STATUSES: [&str; 2] = ["new_status", "in_progress"];
const OVERDUE_DISPLAY_LIMIT: usize = 5;
// Daylite applies its own ordering when truncating server-side, so a wider
// candidate pool keeps the projects with the lowest IDs deterministic.
const OVERDUE_CANDIDATE_LIMIT: u16 = 50;

#[tauri::command]
#[specta::specta]
pub async fn daylite_query_overdue_projects(
    app: tauri::AppHandle,
) -> Result<Vec<DayliteProjectSummary>, DayliteApiError> {
    run_daylite_command(app, |client, tokens| {
        Box::pin(query_overdue_projects_core(client, tokens))
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_search_projects(
    app: tauri::AppHandle,
    input: DayliteSearchInput,
) -> Result<DayliteSearchResult<DayliteProjectSummary>, DayliteApiError> {
    run_daylite_command(app, move |client, tokens| {
        let input = input.clone();
        Box::pin(async move { search_projects_core(client, tokens, &input).await })
    })
    .await
}

pub(super) async fn query_overdue_projects_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
) -> Result<(Vec<DayliteProjectSummary>, DayliteTokenState), DayliteApiError> {
    let clauses: Vec<serde_json::Value> = OVERDUE_STATUSES
        .iter()
        .map(|status| {
            json!({
                "category": { "equal": OVERDUE_CATEGORY },
                "status": { "equal": status }
            })
        })
        .collect();

    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummaryDto>>(
            client,
            token_state,
            DayliteHttpRequest {
                query: build_limit_query(Some(OVERDUE_CANDIDATE_LIMIT)),
                body: Some(json!(clauses)),
                ..DayliteHttpRequest::new(DayliteHttpMethod::Post, "/projects/_search")
            },
        )
        .await?;

    let mut results: Vec<DayliteProjectSummary> = search_result
        .results
        .into_iter()
        .map(normalize_project_summary)
        // The search filters on the overdue category, so every result carries it even
        // though Daylite omits the field from these records.
        .map(|project| DayliteProjectSummary {
            category: Some(OVERDUE_CATEGORY.to_string()),
            ..project
        })
        .collect();
    results.sort_by_key(|project| extract_numeric_id(&project.reference));
    results.truncate(OVERDUE_DISPLAY_LIMIT);

    Ok((results, token_state))
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
    let body = match &input.statuses {
        Some(statuses) if !statuses.is_empty() => {
            let clauses: Vec<serde_json::Value> = statuses
                .iter()
                .map(|s| {
                    json!({
                        "name": { "contains": input.search_term },
                        "status": { "equal": s }
                    })
                })
                .collect();
            json!(clauses)
        }
        _ => json!({ "name": { "contains": input.search_term } }),
    };

    let mut query = build_limit_query(input.limit);
    if input.full_records == Some(true) {
        query.push(("full-records".to_string(), "true".to_string()));
    }
    if let Some(start) = &input.start {
        query.push(("start".to_string(), start.clone()));
    }

    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummaryDto>>(
            client,
            token_state,
            DayliteHttpRequest {
                query,
                body: Some(body),
                ..DayliteHttpRequest::new(DayliteHttpMethod::Post, "/projects/_search")
            },
        )
        .await?;

    let mut results: Vec<DayliteProjectSummary> = search_result
        .results
        .into_iter()
        .map(normalize_project_summary)
        .collect();

    match input.sort {
        Some(DayliteSearchSort::Name) => results.sort_by(|a, b| a.name.cmp(&b.name)),
        _ => results.sort_by_key(|p| extract_numeric_id(&p.reference)),
    }

    if let Some(limit) = input.limit {
        results.truncate(limit as usize);
    }

    Ok((
        DayliteSearchResult {
            results,
            next: trimmed_or_none(search_result.next),
        },
        token_state,
    ))
}

fn extract_numeric_id(reference: &str) -> u64 {
    reference
        .rsplit('/')
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(u64::MAX)
}

fn map_daylite_project_summary(project: DayliteProjectSummaryDto) -> PlanningProjectRecord {
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

fn normalize_project_summary(project: DayliteProjectSummaryDto) -> DayliteProjectSummary {
    DayliteProjectSummary {
        reference: trimmed(project.reference),
        name: trimmed(project.name),
        status: trimmed_or_none(project.status),
        category: trimmed_or_none(project.category),
        keywords: project
            .keywords
            .into_iter()
            .map(trimmed)
            .filter(|keyword| !keyword.is_empty())
            .collect(),
        due: normalize_optional_date(project.due),
        started: normalize_optional_date(project.started),
        completed: normalize_optional_date(project.completed),
        create_date: normalize_optional_date(project.create_date),
        modify_date: normalize_optional_date(project.modify_date),
    }
}

fn normalize_optional_date(value: Option<String>) -> Option<String> {
    let raw_value = trimmed_or_none(value)?;

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
    let normalized = trimmed_or_none(status)
        .map(|value| value.to_lowercase())
        .unwrap_or_default();

    match normalized.as_str() {
        "in_progress" => PlanningProjectStatus::InProgress,
        "done" => PlanningProjectStatus::Done,
        "abandoned" => PlanningProjectStatus::Abandoned,
        "cancelled" => PlanningProjectStatus::Cancelled,
        "deferred" => PlanningProjectStatus::Deferred,
        _ => PlanningProjectStatus::NewStatus,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ResolvedProject {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) category: Option<String>,
}

// An unbounded burst is what a rate limit answers with 429.
const PROJECT_RESOLUTION_CONCURRENCY: usize = 6;

pub(crate) async fn resolve_projects_by_reference(
    daylite_base_url: &str,
    references: Vec<String>,
) -> HashMap<String, Option<ResolvedProject>> {
    let Ok(client) = DayliteApiClient::new(daylite_base_url) else {
        return references
            .into_iter()
            .map(|reference| (reference, None))
            .collect();
    };

    resolve_all(references, PROJECT_RESOLUTION_CONCURRENCY, |reference| {
        let client = &client;
        async move { resolve_project_cached(client, &reference).await }
    })
    .await
}

async fn resolve_all<F, Fut>(
    references: Vec<String>,
    concurrency: usize,
    resolve: F,
) -> HashMap<String, Option<ResolvedProject>>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Option<ResolvedProject>>,
{
    futures::stream::iter(references)
        .map(|reference| {
            let resolve = &resolve;
            async move {
                let project = resolve(reference.clone()).await;
                (reference, project)
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await
}

async fn resolve_project_cached(
    client: &DayliteApiClient,
    project_ref: &str,
) -> Option<ResolvedProject> {
    project_cache()
        .get_or_load(project_ref, cache_now_ms(), || {
            fetch_project(client, project_ref)
        })
        .await
}

/// Deliberately uncached: the fixed-appointment guard must see the category as it stands now.
pub(crate) async fn fetch_project_by_reference(
    app: tauri::AppHandle,
    project_ref: &str,
) -> Option<ResolvedProject> {
    let store = crate::integrations::local_store::load_local_store(app).ok()?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url).ok()?;

    fetch_project(&client, project_ref).await
}

async fn fetch_project(client: &DayliteApiClient, project_ref: &str) -> Option<ResolvedProject> {
    // The project_ref is an absolute API path like "/v1/projects/3001".
    // The DayliteApiClient base_url already includes the version prefix, so strip "/v1".
    let path = project_ref.strip_prefix("/v1").unwrap_or(project_ref);
    if path.is_empty() {
        return None;
    }

    with_daylite_tokens(client, |tokens| async {
        let (summary, tokens): (DayliteProjectSummaryDto, _) = send_authenticated_json(
            client,
            tokens,
            DayliteHttpRequest::new(DayliteHttpMethod::Get, path),
        )
        .await?;
        Ok((resolve_project(summary), tokens))
    })
    .await
    .ok()
}

fn resolve_project(summary: DayliteProjectSummaryDto) -> ResolvedProject {
    let mapped = map_daylite_project_summary(summary);
    ResolvedProject {
        name: mapped.name,
        status: project_status_to_string(&mapped.status).to_string(),
        category: mapped.category,
    }
}

fn project_status_to_string(status: &PlanningProjectStatus) -> &'static str {
    match status {
        PlanningProjectStatus::InProgress => "in_progress",
        PlanningProjectStatus::Done => "done",
        PlanningProjectStatus::Abandoned => "abandoned",
        PlanningProjectStatus::Cancelled => "cancelled",
        PlanningProjectStatus::Deferred => "deferred",
        PlanningProjectStatus::NewStatus => "new_status",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_daylite_project_summary, map_project_status, query_overdue_projects_core, resolve_all,
        resolve_project, search_projects_core, DayliteProjectSummaryDto, PlanningProjectStatus,
        ResolvedProject, FIXED_APPOINTMENT_CATEGORY,
    };
    use crate::integrations::daylite::client::DayliteApiClient;
    use crate::integrations::daylite::client::DayliteHttpMethod;
    use crate::integrations::daylite::shared::{
        DayliteApiError, DayliteApiErrorCode, DayliteSearchInput, DayliteSearchResult,
        DayliteSearchSort,
    };
    use crate::integrations::daylite::test_support::{
        mock_client, mock_response, token_state, valid_token_state,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn resolved(name: &str) -> ResolvedProject {
        ResolvedProject {
            name: name.to_string(),
            status: "in_progress".to_string(),
            category: None,
        }
    }

    #[tokio::test]
    async fn keeps_each_project_against_its_own_reference_when_replies_arrive_out_of_order() {
        let references = vec![
            "/v1/projects/1".to_string(),
            "/v1/projects/2".to_string(),
            "/v1/projects/3".to_string(),
        ];

        let all = resolve_all(references, 3, |reference| async move {
            // The last reference settles first, so the results arrive reversed.
            let delay = 3 - reference
                .rsplit('/')
                .next()
                .unwrap()
                .parse::<usize>()
                .unwrap();
            for _ in 0..delay {
                tokio::task::yield_now().await;
            }
            Some(resolved(&format!("Projekt {reference}")))
        })
        .await;

        assert_eq!(all.len(), 3);
        for reference in ["/v1/projects/1", "/v1/projects/2", "/v1/projects/3"] {
            assert_eq!(
                all.get(reference).unwrap().as_ref().unwrap().name,
                format!("Projekt {reference}")
            );
        }
    }

    #[tokio::test]
    async fn keeps_no_more_than_the_configured_number_of_requests_in_flight() {
        let references: Vec<String> = (1..=12).map(|id| format!("/v1/projects/{id}")).collect();
        let in_flight = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);

        let all = resolve_all(references, 4, |_| {
            let in_flight = &in_flight;
            let peak = &peak;
            async move {
                let running = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                peak.fetch_max(running, Ordering::SeqCst);
                tokio::task::yield_now().await;
                in_flight.fetch_sub(1, Ordering::SeqCst);
                Some(resolved("Projekt"))
            }
        })
        .await;

        assert_eq!(all.len(), 12);
        assert_eq!(peak.load(Ordering::SeqCst), 4);
    }

    #[tokio::test]
    async fn records_a_reference_that_did_not_resolve() {
        let all = resolve_all(vec!["/v1/projects/9999".to_string()], 4, |_| async { None }).await;

        assert_eq!(all.get("/v1/projects/9999"), Some(&None));
    }

    #[test]
    fn resolved_project_carries_the_category_alongside_name_and_status() {
        let cases: &[(Option<&str>, Option<&str>)] = &[
            (Some(FIXED_APPOINTMENT_CATEGORY), Some("Termin FIX geplant")),
            (Some("Liefertermin bekannt"), Some("Liefertermin bekannt")),
            (None, None),
        ];

        for (category, expected) in cases {
            let resolved = resolve_project(DayliteProjectSummaryDto {
                reference: "/v1/projects/3001".to_string(),
                name: "Projekt Nord".to_string(),
                status: Some("in_progress".to_string()),
                category: category.map(str::to_string),
                keywords: vec![],
                due: None,
                started: None,
                completed: None,
                create_date: None,
                modify_date: None,
            });

            assert_eq!(resolved.name, "Projekt Nord");
            assert_eq!(resolved.status, "in_progress");
            assert_eq!(resolved.category.as_deref(), *expected);
        }
    }

    #[test]
    fn maps_project_summary_to_planning_project_record() {
        let project = DayliteProjectSummaryDto {
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

    #[tokio::test]
    async fn search_projects_sends_correct_body_and_query() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[{"self":" /v1/projects/10 ","name":" Projekt Nord ","category":" Bau ","keywords":[" Aufträge ",""],"due":"2026-02-15"}],"next":" /v1/projects/_search?offset=5 "}"#,
        ))]);

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
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
    }

    #[tokio::test]
    async fn search_results_are_sorted_by_numeric_id_ascending() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[
            {"self":"/v1/projects/100","name":"Hundert"},
            {"self":"/v1/projects/20","name":"Zwanzig"},
            {"self":"/v1/projects/3","name":"Drei"}
        ],"next":null}"#,
        ))]);

        let (result, _) =
            search_projects_core(&client, valid_token_state(), &DayliteSearchInput::default())
                .await
                .expect("search should succeed");

        assert_eq!(result.results[0].reference, "/v1/projects/3");
        assert_eq!(result.results[1].reference, "/v1/projects/20");
        assert_eq!(result.results[2].reference, "/v1/projects/100");
    }

    #[test]
    fn empty_object_response_deserializes_as_no_results() {
        let result: DayliteSearchResult<DayliteProjectSummaryDto> =
            serde_json::from_str(r#"{}"#).expect("a bare object is a valid empty search result");

        assert!(result.results.is_empty());
        assert_eq!(result.next, None);
    }

    #[tokio::test]
    async fn search_sorts_by_name_when_sort_is_name() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[
            {"self":"/v1/projects/1","name":"Zeta"},
            {"self":"/v1/projects/2","name":"Alpha"},
            {"self":"/v1/projects/3","name":"Mitte"}
        ],"next":null}"#,
        ))]);

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                sort: Some(DayliteSearchSort::Name),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        assert_eq!(result.results[0].name, "Alpha");
        assert_eq!(result.results[1].name, "Mitte");
        assert_eq!(result.results[2].name, "Zeta");
    }

    #[tokio::test]
    async fn search_limit_is_applied_after_sort() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[
            {"self":"/v1/projects/100","name":"Hundert"},
            {"self":"/v1/projects/20","name":"Zwanzig"},
            {"self":"/v1/projects/3","name":"Drei"}
        ],"next":null}"#,
        ))]);

        let (result, _) = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                limit: Some(2),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        assert_eq!(result.results.len(), 2);
        assert_eq!(result.results[0].reference, "/v1/projects/3");
        assert_eq!(result.results[1].reference, "/v1/projects/20");
    }

    #[tokio::test]
    async fn overdue_query_sends_category_and_status_filter_in_a_single_call() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);

        query_overdue_projects_core(&client, valid_token_state())
            .await
            .expect("overdue query should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1, "overdue query must be a single call");
        assert_eq!(requests[0].path, "/projects/_search");
        assert_eq!(requests[0].method, DayliteHttpMethod::Post);
        let body = requests[0].body.as_ref().expect("body should be present");
        assert_eq!(
            *body,
            serde_json::json!([
                {
                    "category": { "equal": "Überfällig" },
                    "status": { "equal": "new_status" }
                },
                {
                    "category": { "equal": "Überfällig" },
                    "status": { "equal": "in_progress" }
                }
            ]),
            "body must pair the category filter with each allowed status as OR clauses"
        );
    }

    #[tokio::test]
    async fn overdue_results_are_sorted_by_numeric_id_and_limited_to_five() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[
            {"self":"/v1/projects/100","name":"Hundert"},
            {"self":"/v1/projects/20","name":"Zwanzig"},
            {"self":"/v1/projects/3","name":"Drei"},
            {"self":"/v1/projects/50","name":"Fünfzig"},
            {"self":"/v1/projects/7","name":"Sieben"},
            {"self":"/v1/projects/9","name":"Neun"}
        ],"next":null}"#,
        ))]);

        let (results, _) = query_overdue_projects_core(&client, valid_token_state())
            .await
            .expect("overdue query should succeed");

        assert_eq!(results.len(), 5);
        let references: Vec<&str> = results
            .iter()
            .map(|project| project.reference.as_str())
            .collect();
        assert_eq!(
            references,
            vec![
                "/v1/projects/3",
                "/v1/projects/7",
                "/v1/projects/9",
                "/v1/projects/20",
                "/v1/projects/50"
            ]
        );
    }

    #[tokio::test]
    async fn overdue_results_carry_the_overdue_category() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[{"self":"/v1/projects/3","name":"Drei"}],"next":null}"#,
        ))]);

        let (results, _) = query_overdue_projects_core(&client, valid_token_state())
            .await
            .expect("overdue query should succeed");

        assert_eq!(results[0].category.as_deref(), Some("Überfällig"));
    }

    #[tokio::test]
    async fn query_overdue_projects_replays_vcr_cassette() {
        // The cassette is produced by the live recording harness (`record_daylite_cassettes_from_live_api`), which needs real Daylite credentials.
        // Skip instead of failing until it has been recorded.
        let cassette_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/cassettes/daylite-overdue-projects.json");
        if !cassette_path.exists() {
            eprintln!(
                "skipping query_overdue_projects_replays_vcr_cassette: cassette not recorded yet"
            );
            return;
        }

        let client = DayliteApiClient::with_replay_cassette("daylite-overdue-projects.json")
            .expect("replay client should be created");

        let (results, token_state) = query_overdue_projects_core(
            &client,
            token_state("replay-access-token", "replay-refresh-token"),
        )
        .await
        .expect("overdue query should replay from cassette");

        assert!(results.len() <= 5);
        assert!(results.iter().all(|project| {
            project.reference.starts_with("/v1/projects/")
                && !project.name.is_empty()
                && project.name == project.name.trim()
        }));
        assert_eq!(token_state.access_token, "replay-access-token");
    }

    #[test]
    fn extract_numeric_id_handles_standard_reference() {
        assert_eq!(super::extract_numeric_id("/v1/projects/3001"), 3001);
        assert_eq!(super::extract_numeric_id("/v1/projects/100"), 100);
        assert_eq!(super::extract_numeric_id("/v1/projects/20"), 20);
    }

    #[test]
    fn extract_numeric_id_returns_max_for_non_numeric() {
        assert_eq!(super::extract_numeric_id("/v1/projects/abc"), u64::MAX);
        assert_eq!(super::extract_numeric_id(""), u64::MAX);
    }

    #[tokio::test]
    async fn search_projects_replays_vcr_cassette() {
        let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
            .expect("replay client should be created");

        let (search_result, token_state) = search_projects_core(
            &client,
            token_state("replay-access-token", "replay-refresh-token"),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
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
        assert!(search_result
            .next
            .as_deref()
            .map(|next| next.starts_with("/v1/projects/_search"))
            .unwrap_or(true));
        assert_eq!(token_state.access_token, "replay-access-token");
    }

    #[tokio::test]
    async fn search_projects_with_status_filter_replays_vcr_cassette() {
        let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
            .expect("status-filter cassette client should be created");

        let (search_result, token_state) = search_projects_core(
            &client,
            token_state("test-token", "test-refresh"),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                full_records: Some(true),
                statuses: Some(vec!["new_status".to_string(), "in_progress".to_string()]),
                ..Default::default()
            },
        )
        .await
        .expect("search with status filter should replay from cassette");

        assert!(
            !search_result.results.is_empty(),
            "cassette should contain results"
        );
        assert_eq!(token_state.access_token, "test-token");

        for project in &search_result.results {
            assert!(
                project.status.as_deref() == Some("new")
                    || project.status.as_deref() == Some("in_progress"),
                "project {:?} has unexpected status",
                project.reference
            );
        }
    }

    #[tokio::test]
    async fn search_projects_no_match_replays_vcr_cassette() {
        let client = DayliteApiClient::with_replay_cassette("daylite-search-projects.json")
            .expect("no-match cassette client should be created");

        let (search_result, token_state) = search_projects_core(
            &client,
            token_state("test-token", "test-refresh"),
            &DayliteSearchInput {
                search_term: "XXXXX".to_string(),
                limit: Some(50),
                statuses: Some(vec!["new_status".to_string(), "in_progress".to_string()]),
                sort: Some(DayliteSearchSort::Name),
                ..Default::default()
            },
        )
        .await
        .expect("no-match search should replay from cassette");

        assert!(search_result.results.is_empty());
        assert_eq!(token_state.access_token, "test-token");
    }

    #[tokio::test]
    async fn search_with_statuses_sends_array_body_with_or_clauses() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                statuses: Some(vec!["new_status".to_string(), "in_progress".to_string()]),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = requests[0].body.as_ref().expect("body should be present");
        assert!(body.is_array(), "body should be an array for OR conditions");
        let items = body.as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"]["contains"], "Nord");
        assert_eq!(items[0]["status"]["equal"], "new_status");
        assert_eq!(items[1]["name"]["contains"], "Nord");
        assert_eq!(items[1]["status"]["equal"], "in_progress");
    }

    #[tokio::test]
    async fn search_without_statuses_sends_plain_object_body() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        let body = requests[0].body.as_ref().expect("body should be present");
        assert!(
            body.is_object(),
            "body should be a plain object when no statuses"
        );
        assert_eq!(body["name"]["contains"], "Nord");
        assert!(
            body.get("status").is_none(),
            "no status key when statuses is None"
        );
    }

    #[tokio::test]
    async fn search_with_full_records_sends_query_param() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                limit: Some(5),
                full_records: Some(true),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert!(
            requests[0]
                .query
                .contains(&("full-records".to_string(), "true".to_string())),
            "query should include full-records=true, got {:?}",
            requests[0].query
        );
    }

    #[tokio::test]
    async fn search_without_full_records_omits_query_param() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert!(
            !requests[0].query.iter().any(|(k, _)| k == "full-records"),
            "query should not include full-records when None, got {:?}",
            requests[0].query
        );
    }

    #[tokio::test]
    async fn search_with_start_sends_query_param() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[],"next":null}"#,
        ))]);

        search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                start: Some("3001".to_string()),
                ..Default::default()
            },
        )
        .await
        .expect("search should succeed");

        let requests = transport.requests();
        assert!(
            requests[0]
                .query
                .contains(&("start".to_string(), "3001".to_string())),
            "query should include start=3001, got {:?}",
            requests[0].query
        );
    }

    #[tokio::test]
    async fn malformed_response_returns_invalid_response_with_german_message() {
        let (client, _) = mock_client(vec![Ok(mock_response(200, "not valid json {{{"))]);

        let result = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                ..Default::default()
            },
        )
        .await;

        let err = result.expect_err("malformed response should return error");
        assert_eq!(err.code, DayliteApiErrorCode::InvalidResponse);
        assert!(
            err.user_message.contains("Daylite"),
            "error message should mention Daylite: {}",
            err.user_message
        );
    }

    #[tokio::test]
    async fn timeout_error_propagates_from_transport() {
        let (client, _) = mock_client(vec![Err(DayliteApiError {
            code: DayliteApiErrorCode::Timeout,
            http_status: None,
            user_message: "Zeitüberschreitung bei der Daylite-Anfrage.".to_string(),
            technical_message: "request timed out".to_string(),
        })]);

        let result = search_projects_core(
            &client,
            valid_token_state(),
            &DayliteSearchInput {
                search_term: "Nord".to_string(),
                ..Default::default()
            },
        )
        .await;

        let err = result.expect_err("timeout from transport should propagate as error");
        assert_eq!(err.code, DayliteApiErrorCode::Timeout);
        assert_eq!(
            err.user_message,
            "Zeitüberschreitung bei der Daylite-Anfrage."
        );
    }
}
