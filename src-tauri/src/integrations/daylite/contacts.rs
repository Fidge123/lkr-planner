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
    #[serde(default, alias = "first_name")]
    pub first_name: String,
    #[serde(default, alias = "last_name")]
    pub last_name: String,
    #[serde(default, alias = "full_name")]
    pub full_name: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub urls: Vec<DayliteContactUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteContactUrl {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DayliteUpdateContactIcalUrlsInput {
    pub contact_reference: String,
    pub primary_ical_url: String,
    pub absence_ical_url: String,
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
    let monteur_count = response
        .data
        .iter()
        .filter(|contact| {
            contact
                .category
                .as_deref()
                .map(str::trim)
                .map(|category| category.eq_ignore_ascii_case("monteur"))
                .unwrap_or(false)
        })
        .count();
    let empty_name_count = response
        .data
        .iter()
        .filter(|contact| {
            let has_full_name = contact
                .full_name
                .as_deref()
                .map(str::trim)
                .map(|value| !value.is_empty())
                .unwrap_or(false);
            let has_nickname = contact
                .nickname
                .as_deref()
                .map(str::trim)
                .map(|value| !value.is_empty())
                .unwrap_or(false);

            !has_full_name
                && !has_nickname
                && contact.first_name.trim().is_empty()
                && contact.last_name.trim().is_empty()
        })
        .count();
    let sample = response
        .data
        .iter()
        .take(5)
        .map(|contact| {
            (
                contact.reference.clone(),
                contact.full_name.clone(),
                contact.first_name.clone(),
                contact.last_name.clone(),
                contact.nickname.clone(),
                contact.category.clone(),
                contact.urls.len(),
            )
        })
        .collect::<Vec<_>>();
    println!(
        "[daylite-contacts] daylite_list_contacts loaded={} monteur={} empty_names={} sample={sample:?}",
        response.data.len(),
        monteur_count,
        empty_name_count,
    );

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

#[tauri::command]
#[specta::specta]
pub async fn daylite_update_contact_ical_urls(
    app: tauri::AppHandle,
    input: DayliteUpdateContactIcalUrlsInput,
) -> Result<DayliteContactSummary, DayliteApiError> {
    println!(
        "[daylite-contacts] daylite_update_contact_ical_urls reference={} primary={} absence={}",
        input.contact_reference, input.primary_ical_url, input.absence_ical_url
    );

    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let token_state = load_daylite_tokens(&store);

    let response = client
        .update_contact_ical_urls(
            token_state,
            &input.contact_reference,
            &input.primary_ical_url,
            &input.absence_ical_url,
        )
        .await?;
    println!(
        "[daylite-contacts] daylite_update_contact_ical_urls updated reference={} urls={}",
        response.data.reference,
        response.data.urls.len()
    );
    store_daylite_tokens(&mut store, &response.token_state);
    save_store_or_error(app, store)?;

    Ok(response.data)
}
