use super::client::DayliteApiClient;
use super::shared::{
    load_daylite_tokens, load_store_or_error, save_store_or_error, store_daylite_tokens,
    DayliteApiError, DayliteSearchInput, DayliteSearchResult,
};
use serde::{Deserialize, Serialize};
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

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_projects(
    app: tauri::AppHandle,
) -> Result<Vec<DayliteProjectSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
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
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let token_state = load_daylite_tokens(&store);

    let response = client
        .search_projects(token_state, &input.search_term, input.limit)
        .await?;
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}
