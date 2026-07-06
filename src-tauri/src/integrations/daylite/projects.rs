use super::auth_flow::send_authenticated_json;
use super::client::DayliteApiClient;
use super::client::DayliteHttpMethod;
use super::client::DayliteHttpRequest;
use super::shared::{
    build_limit_query, load_store_or_error, save_store_or_error, with_token_refresh_lock,
    DayliteApiError, DayliteSearchInput, DayliteSearchResult, DayliteSearchSort, DayliteTokenState,
};
use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use specta::Type;

// Raw project record as returned by the Daylite API. Only `self` needs a serde
// rename (Rust keyword); the other snake_case field names match Daylite directly.
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

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_projects(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningProjectRecord>, DayliteApiError> {
    let store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let projects = with_token_refresh_lock(|tokens| list_projects_core(&client, tokens)).await?;

    save_store_or_error(app, store)?;

    Ok(projects)
}

const OVERDUE_CATEGORY: &str = "Überfällig";
// The Daylite API has no multi-value operator for scalar fields, so the overdue
// query pairs the category filter with each status as OR clauses to stay a
// single call. The statuses match the assignment picker's search filter.
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
    let store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let projects =
        with_token_refresh_lock(|tokens| query_overdue_projects_core(&client, tokens)).await?;

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
    let search_result =
        with_token_refresh_lock(|tokens| search_projects_core(&client, tokens, &input)).await?;

    save_store_or_error(app, store)?;

    Ok(search_result)
}

pub(super) async fn list_projects_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
) -> Result<(Vec<PlanningProjectRecord>, DayliteTokenState), DayliteApiError> {
    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummaryDto>>(
            client,
            token_state,
            DayliteHttpRequest {
                query: vec![("full-records".to_string(), "true".to_string())],
                body: Some(json!({})),
                ..DayliteHttpRequest::new(DayliteHttpMethod::Post, "/projects/_search")
            },
        )
        .await?;

    let projects = search_result
        .results
        .into_iter()
        .map(map_daylite_project_summary)
        .collect();

    Ok((projects, token_state))
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
            next: normalize_optional_string(search_result.next),
        },
        token_state,
    ))
}

/// Extracts the trailing integer from a Daylite reference path like `/v1/projects/3001`.
/// Returns `u64::MAX` for references that don't end with a numeric ID so they sort last.
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
        reference: normalize_required_string(project.reference),
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

/// Fetches a single project by its Daylite reference (e.g. "/v1/projects/3001") and returns
/// `(name, status_string)`. Returns `None` on any error so callers can show a placeholder instead.
/// Intended for use as a cache fallback in other integrations.
pub(crate) async fn fetch_project_by_reference(
    app: tauri::AppHandle,
    project_ref: &str,
) -> Option<(String, String)> {
    // The project_ref is an absolute API path like "/v1/projects/3001".
    // The DayliteApiClient base_url already includes the version prefix, so strip "/v1".
    let path = project_ref.strip_prefix("/v1").unwrap_or(project_ref);
    if path.is_empty() {
        return None;
    }

    let store = crate::integrations::local_store::load_local_store(app).ok()?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url).ok()?;

    with_token_refresh_lock(|tokens| async move {
        let (summary, tokens): (DayliteProjectSummaryDto, _) = send_authenticated_json(
            &client,
            tokens,
            DayliteHttpRequest::new(DayliteHttpMethod::Get, path),
        )
        .await?;
        let mapped = map_daylite_project_summary(summary);
        let status_str = project_status_to_string(&mapped.status);
        Ok(((mapped.name, status_str.to_string()), tokens))
    })
    .await
    .ok()
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
mod tests;
