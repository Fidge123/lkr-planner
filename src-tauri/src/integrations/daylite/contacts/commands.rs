use super::api::{current_timestamp_iso8601, list_contacts_core, update_contact_ical_urls_core};
use super::ical_urls::reconcile_employee_calendars_from_contacts;
use super::mapping::{
    filter_planning_contacts, map_cached_contact, map_planning_contact_to_cache_entry,
    sort_contacts,
};
use super::types::{DayliteUpdateContactIcalUrlsInput, PlanningContactRecord};
use crate::integrations::daylite::client::DayliteApiClient;
use crate::integrations::daylite::shared::{
    load_store_or_error, save_store_or_error, with_token_refresh_lock, DayliteApiError,
};

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let contacts = with_token_refresh_lock(|tokens| list_contacts_core(&client, tokens)).await?;

    store.daylite_cache.last_synced_at = Some(current_timestamp_iso8601());
    store.daylite_cache.contacts = contacts
        .iter()
        .cloned()
        .map(map_planning_contact_to_cache_entry)
        .collect();
    reconcile_employee_calendars_from_contacts(&mut store.employee_settings, &contacts);
    crate::integrations::zep::test_untested_calendar_urls(&mut store.employee_settings).await;
    save_store_or_error(app, store)?;

    Ok(contacts)
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_update_contact_ical_urls(
    app: tauri::AppHandle,
    input: DayliteUpdateContactIcalUrlsInput,
) -> Result<PlanningContactRecord, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;

    let updated_contact = with_token_refresh_lock(|tokens| {
        update_contact_ical_urls_core(&client, tokens, &mut store, &input)
    })
    .await?;

    store.daylite_cache.last_synced_at = Some(current_timestamp_iso8601());
    save_store_or_error(app, store)?;

    Ok(updated_contact)
}

#[tauri::command]
#[specta::specta]
pub fn daylite_list_cached_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, DayliteApiError> {
    let store = load_store_or_error(app)?;
    Ok(sort_contacts(filter_planning_contacts(
        store
            .daylite_cache
            .contacts
            .into_iter()
            .map(map_cached_contact)
            .collect(),
    )))
}
