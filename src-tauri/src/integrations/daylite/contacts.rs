use super::client::DayliteApiClient;
use super::shared::{
    load_daylite_tokens, load_store_or_error, save_store_or_error, store_daylite_tokens,
    DayliteApiError, DayliteSearchInput, DayliteSearchResult,
};
use serde::{Deserialize, Serialize};
use specta::Type;

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

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<DayliteContactSummary>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
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
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let token_state = load_daylite_tokens(&store);

    let response = client
        .search_contacts(token_state, &input.search_term, input.limit)
        .await?;
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}
