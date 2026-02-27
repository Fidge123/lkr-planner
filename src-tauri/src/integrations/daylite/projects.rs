use super::auth_flow::send_authenticated_json;
use super::client::DayliteApiClient;
use super::client::DayliteHttpMethod;
use super::shared::{
    build_limit_query, load_daylite_tokens, load_store_or_error, save_store_or_error,
    store_daylite_tokens, DayliteApiError, DayliteSearchInput, DayliteSearchResult,
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
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummary>>(
            &client,
            load_daylite_tokens(&store),
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

    store_daylite_tokens(&mut store, &token_state);
    save_store_or_error(app, store)?;

    Ok(projects)
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_search_projects(
    app: tauri::AppHandle,
    input: DayliteSearchInput,
) -> Result<DayliteSearchResult<DayliteProjectSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let (search_result, token_state) =
        send_authenticated_json::<DayliteSearchResult<DayliteProjectSummary>>(
            &client,
            load_daylite_tokens(&store),
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

    store_daylite_tokens(&mut store, &token_state);
    save_store_or_error(app, store)?;

    Ok(search_result)
}

fn map_daylite_project_summary(project: DayliteProjectSummary) -> PlanningProjectRecord {
    PlanningProjectRecord {
        reference: normalize_reference(project.reference),
        name: project.name,
        status: map_project_status(project.status),
        category: normalize_optional_string(project.category),
        keywords: normalize_keywords(project.keywords),
        due: normalize_optional_date(project.due),
        started: normalize_optional_date(project.started),
        completed: normalize_optional_date(project.completed),
        create_date: normalize_optional_date(project.create_date),
        modify_date: normalize_optional_date(project.modify_date),
    }
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

fn normalize_reference(value: String) -> String {
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

#[cfg(test)]
mod tests {
    use super::{
        map_daylite_project_summary, map_project_status, DayliteProjectSummary,
        PlanningProjectStatus,
    };

    #[test]
    fn maps_project_summary_to_planning_project_record() {
        let project = DayliteProjectSummary {
            reference: "/v1/projects/7000".to_string(),
            name: "Projekt Nord".to_string(),
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
}
