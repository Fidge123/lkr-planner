use super::mapping::{filter_planning_contacts, map_cached_contact, sort_contacts};
use super::types::PlanningContactRecord;
use crate::error::AppError;
use crate::integrations::daylite::shared::load_store_or_error;
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;

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
