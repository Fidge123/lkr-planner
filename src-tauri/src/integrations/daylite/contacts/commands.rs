use super::api::{current_timestamp_iso8601, list_contacts_core};
use super::ical_urls::reconcile_employee_calendars_from_contacts;
use super::mapping::{
    filter_planning_contacts, map_cached_contact, map_planning_contact_to_cache_entry,
    sort_contacts,
};
use super::types::PlanningContactRecord;
use crate::error::AppError;
use crate::integrations::daylite::client::DayliteApiClient;
use crate::integrations::daylite::shared::{
    load_store_or_error, update_store_or_error, with_daylite_tokens,
};
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, AppError> {
    observe(
        &app,
        Operation::DayliteListContacts,
        Integration::Daylite,
        async {
            let base_url = load_store_or_error(app.clone())?
                .api_endpoints
                .daylite_base_url;
            let client = DayliteApiClient::new(&base_url)?.with_telemetry(&app);
            let contacts =
                with_daylite_tokens(&client, |tokens| list_contacts_core(&client, tokens)).await?;

            let mut settings = load_store_or_error(app.clone())?.employee_settings;
            reconcile_employee_calendars_from_contacts(&mut settings, &contacts);
            crate::integrations::zep::test_untested_calendar_urls(&mut settings).await;
            update_store_or_error(&app, |store| {
                store.daylite_cache.last_synced_at = Some(current_timestamp_iso8601());
                store.daylite_cache.contacts = contacts
                    .iter()
                    .cloned()
                    .map(map_planning_contact_to_cache_entry)
                    .collect();
                store.employee_settings = settings;
            })?;

            Ok(contacts)
        },
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_cached_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, AppError> {
    observe(
        &app,
        Operation::DayliteListCachedContacts,
        Integration::Daylite,
        async {
            let store = load_store_or_error(app.clone())?;
            Ok(sort_contacts(filter_planning_contacts(
                store
                    .daylite_cache
                    .contacts
                    .into_iter()
                    .map(map_cached_contact)
                    .collect(),
            )))
        },
    )
    .await
}
