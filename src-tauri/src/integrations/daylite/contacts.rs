use super::super::local_store::{DayliteContactCacheEntry, DayliteContactUrlCacheEntry};
use super::client::DayliteApiClient;
use super::shared::{
    load_daylite_tokens, load_store_or_error, save_store_or_error, store_daylite_tokens,
    DayliteApiError,
};
use chrono::{SecondsFormat, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct PlanningContactRecord {
    #[serde(rename = "self")]
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub urls: Vec<DayliteContactUrl>,
}

#[tauri::command]
#[specta::specta]
pub async fn daylite_list_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, DayliteApiError> {
    let mut store = load_store_or_error(app.clone())?;
    let client = DayliteApiClient::new(&store.api_endpoints.daylite_base_url)?;
    let token_state = load_daylite_tokens(&store);

    let response = client.list_contacts(token_state).await?;
    let contacts = sort_contacts(filter_monteur_contacts(
        response
            .data
            .into_iter()
            .map(map_daylite_contact_summary)
            .collect(),
    ));

    store_daylite_tokens(&mut store, &response.token_state);
    store.daylite_cache.last_synced_at = Some(current_timestamp_iso8601());
    store.daylite_cache.contacts = contacts
        .iter()
        .cloned()
        .map(map_planning_contact_to_cache_entry)
        .collect();
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
    let token_state = load_daylite_tokens(&store);

    let response = client
        .update_contact_ical_urls(
            token_state,
            &input.contact_reference,
            &input.primary_ical_url,
            &input.absence_ical_url,
        )
        .await?;

    let updated_contact = map_daylite_contact_summary(response.data);
    let mut cached_contacts: Vec<PlanningContactRecord> = store
        .daylite_cache
        .contacts
        .clone()
        .into_iter()
        .map(map_cached_contact)
        .collect();

    cached_contacts.retain(|contact| contact.reference != updated_contact.reference);
    if is_monteur_contact(&updated_contact) {
        cached_contacts.push(updated_contact.clone());
    }

    store_daylite_tokens(&mut store, &response.token_state);
    store.daylite_cache.last_synced_at = Some(current_timestamp_iso8601());
    store.daylite_cache.contacts = sort_contacts(filter_monteur_contacts(cached_contacts))
        .into_iter()
        .map(map_planning_contact_to_cache_entry)
        .collect();
    save_store_or_error(app, store)?;

    Ok(updated_contact)
}

#[tauri::command]
#[specta::specta]
pub fn daylite_list_cached_contacts(
    app: tauri::AppHandle,
) -> Result<Vec<PlanningContactRecord>, DayliteApiError> {
    let store = load_store_or_error(app)?;
    Ok(sort_contacts(filter_monteur_contacts(
        store
            .daylite_cache
            .contacts
            .into_iter()
            .map(map_cached_contact)
            .collect(),
    )))
}

fn map_daylite_contact_summary(contact: DayliteContactSummary) -> PlanningContactRecord {
    let full_name = normalize_optional_string(contact.full_name)
        .or_else(|| join_name(&contact.first_name, &contact.last_name));

    PlanningContactRecord {
        reference: normalize_string(contact.reference),
        full_name,
        nickname: normalize_optional_string(contact.nickname),
        category: normalize_optional_string(contact.category),
        urls: normalize_contact_urls(contact.urls),
    }
}

fn map_cached_contact(contact: DayliteContactCacheEntry) -> PlanningContactRecord {
    PlanningContactRecord {
        reference: normalize_string(contact.reference),
        full_name: normalize_optional_string(contact.full_name),
        nickname: normalize_optional_string(contact.nickname),
        category: normalize_optional_string(contact.category),
        urls: normalize_cached_contact_urls(contact.urls),
    }
}

fn map_planning_contact_to_cache_entry(contact: PlanningContactRecord) -> DayliteContactCacheEntry {
    DayliteContactCacheEntry {
        reference: contact.reference,
        full_name: contact.full_name,
        nickname: contact.nickname,
        category: contact.category,
        urls: contact
            .urls
            .into_iter()
            .map(|url| DayliteContactUrlCacheEntry {
                label: url.label,
                url: url.url,
                note: url.note,
            })
            .collect(),
    }
}

fn filter_monteur_contacts(contacts: Vec<PlanningContactRecord>) -> Vec<PlanningContactRecord> {
    contacts
        .into_iter()
        .filter(is_monteur_contact)
        .collect::<Vec<_>>()
}

fn is_monteur_contact(contact: &PlanningContactRecord) -> bool {
    normalize_string_option(contact.category.clone())
        .map(|category| category.to_lowercase() == "monteur")
        .unwrap_or(false)
}

fn sort_contacts(mut contacts: Vec<PlanningContactRecord>) -> Vec<PlanningContactRecord> {
    contacts.sort_by(|left_contact, right_contact| {
        contact_display_name(left_contact)
            .to_lowercase()
            .cmp(&contact_display_name(right_contact).to_lowercase())
    });
    contacts
}

fn contact_display_name(contact: &PlanningContactRecord) -> String {
    if let Some(nickname) = normalize_string_option(contact.nickname.clone()) {
        return nickname;
    }

    if let Some(full_name) = normalize_string_option(contact.full_name.clone()) {
        return full_name;
    }

    "Unbenannter Kontakt".to_string()
}

fn normalize_contact_urls(urls: Vec<DayliteContactUrl>) -> Vec<DayliteContactUrl> {
    urls.into_iter()
        .filter_map(|url| {
            let normalized_url = DayliteContactUrl {
                label: normalize_optional_string(url.label),
                url: normalize_optional_string(url.url),
                note: normalize_optional_string(url.note),
            };

            if normalized_url.label.is_none()
                && normalized_url.url.is_none()
                && normalized_url.note.is_none()
            {
                None
            } else {
                Some(normalized_url)
            }
        })
        .collect()
}

fn normalize_cached_contact_urls(urls: Vec<DayliteContactUrlCacheEntry>) -> Vec<DayliteContactUrl> {
    urls.into_iter()
        .filter_map(|url| {
            let normalized_url = DayliteContactUrl {
                label: normalize_optional_string(url.label),
                url: normalize_optional_string(url.url),
                note: normalize_optional_string(url.note),
            };

            if normalized_url.label.is_none()
                && normalized_url.url.is_none()
                && normalized_url.note.is_none()
            {
                None
            } else {
                Some(normalized_url)
            }
        })
        .collect()
}

fn normalize_string(value: String) -> String {
    value.trim().to_string()
}

fn normalize_string_option(value: Option<String>) -> Option<String> {
    value.and_then(|candidate| {
        let normalized = candidate.trim();
        if normalized.is_empty() {
            None
        } else {
            Some(normalized.to_string())
        }
    })
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    normalize_string_option(value)
}

fn join_name(first_name: &str, last_name: &str) -> Option<String> {
    let normalized_first_name = first_name.trim();
    let normalized_last_name = last_name.trim();
    let full_name = [normalized_first_name, normalized_last_name]
        .iter()
        .filter(|value| !value.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ");

    if full_name.is_empty() {
        None
    } else {
        Some(full_name)
    }
}

fn current_timestamp_iso8601() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

#[cfg(test)]
mod tests {
    use super::{
        filter_monteur_contacts, map_cached_contact, map_daylite_contact_summary, sort_contacts,
        DayliteContactSummary, DayliteContactUrl, PlanningContactRecord,
    };
    use crate::integrations::local_store::{DayliteContactCacheEntry, DayliteContactUrlCacheEntry};

    #[test]
    fn maps_daylite_contact_summary_to_planning_contact_record() {
        let contact = DayliteContactSummary {
            reference: "/v1/contacts/1001".to_string(),
            first_name: " Max ".to_string(),
            last_name: " Mustermann ".to_string(),
            full_name: None,
            nickname: Some("  ".to_string()),
            category: Some(" Monteur ".to_string()),
            urls: vec![
                DayliteContactUrl {
                    label: Some(" Einsatz iCal ".to_string()),
                    url: Some(" https://example.com/max-primary.ics ".to_string()),
                    note: None,
                },
                DayliteContactUrl {
                    label: Some(" ".to_string()),
                    url: None,
                    note: None,
                },
            ],
        };

        let mapped = map_daylite_contact_summary(contact);

        assert_eq!(mapped.reference, "/v1/contacts/1001");
        assert_eq!(mapped.full_name, Some("Max Mustermann".to_string()));
        assert_eq!(mapped.nickname, None);
        assert_eq!(mapped.category, Some("Monteur".to_string()));
        assert_eq!(
            mapped.urls,
            vec![DayliteContactUrl {
                label: Some("Einsatz iCal".to_string()),
                url: Some("https://example.com/max-primary.ics".to_string()),
                note: None,
            }]
        );
    }

    #[test]
    fn maps_cached_contact_without_display_name_fallback() {
        let cached_contact = DayliteContactCacheEntry {
            reference: "/v1/contacts/2001".to_string(),
            full_name: None,
            nickname: None,
            category: Some("Monteur".to_string()),
            urls: vec![DayliteContactUrlCacheEntry {
                label: Some("Abwesenheit iCal".to_string()),
                url: Some("https://example.com/moritz-absence.ics".to_string()),
                note: None,
            }],
        };

        let mapped = map_cached_contact(cached_contact);

        assert_eq!(mapped.reference, "/v1/contacts/2001");
        assert_eq!(mapped.full_name, None);
        assert_eq!(mapped.category, Some("Monteur".to_string()));
        assert_eq!(
            mapped.urls,
            vec![DayliteContactUrl {
                label: Some("Abwesenheit iCal".to_string()),
                url: Some("https://example.com/moritz-absence.ics".to_string()),
                note: None,
            }]
        );
    }

    #[test]
    fn filters_and_sorts_monteur_contacts_by_display_name() {
        let contacts = vec![
            PlanningContactRecord {
                reference: "/v1/contacts/3001".to_string(),
                full_name: Some("Zora Monteur".to_string()),
                nickname: None,
                category: Some("Monteur".to_string()),
                urls: vec![],
            },
            PlanningContactRecord {
                reference: "/v1/contacts/3002".to_string(),
                full_name: Some("Anna Vertrieb".to_string()),
                nickname: None,
                category: Some("Vertrieb".to_string()),
                urls: vec![],
            },
            PlanningContactRecord {
                reference: "/v1/contacts/3003".to_string(),
                full_name: Some("Max Mustermann".to_string()),
                nickname: Some("Maks".to_string()),
                category: Some("Monteur".to_string()),
                urls: vec![],
            },
        ];

        let mapped = sort_contacts(filter_monteur_contacts(contacts));

        assert_eq!(mapped.len(), 2);
        assert_eq!(mapped[0].reference, "/v1/contacts/3003");
        assert_eq!(mapped[1].reference, "/v1/contacts/3001");
    }
}
