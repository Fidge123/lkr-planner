use super::types::{DayliteContactSummary, PlanningContactRecord};
use crate::integrations::local_store::{DayliteContactCacheEntry, DayliteContactUrl};

pub(super) fn map_daylite_contact_summary(contact: DayliteContactSummary) -> PlanningContactRecord {
    let full_name = normalize_string_option(contact.full_name)
        .or_else(|| join_name(&contact.first_name, &contact.last_name));

    PlanningContactRecord {
        reference: normalize_string(contact.reference),
        full_name,
        nickname: normalize_string_option(contact.nickname),
        category: normalize_string_option(contact.category),
        urls: normalize_contact_urls(contact.urls),
    }
}

pub(super) fn map_cached_contact(contact: DayliteContactCacheEntry) -> PlanningContactRecord {
    PlanningContactRecord {
        reference: normalize_string(contact.reference),
        full_name: normalize_string_option(contact.full_name),
        nickname: normalize_string_option(contact.nickname),
        category: normalize_string_option(contact.category),
        urls: normalize_contact_urls(contact.urls),
    }
}

pub(super) fn map_planning_contact_to_cache_entry(
    contact: PlanningContactRecord,
) -> DayliteContactCacheEntry {
    DayliteContactCacheEntry {
        reference: contact.reference,
        full_name: contact.full_name,
        nickname: contact.nickname,
        category: contact.category,
        urls: contact.urls,
    }
}

/// Keeps only contacts relevant to the planning view. This covers both planning
/// categories — "Monteur" and "Test" — because "Test" employees are fetched too
/// and only hidden in the frontend when the "hide non-plannable employees"
/// toggle is enabled.
pub(super) fn filter_planning_contacts(
    contacts: Vec<PlanningContactRecord>,
) -> Vec<PlanningContactRecord> {
    contacts
        .into_iter()
        .filter(is_planning_contact)
        .collect::<Vec<_>>()
}

pub(super) fn is_planning_contact(contact: &PlanningContactRecord) -> bool {
    normalize_string_option(contact.category.clone())
        .map(|category| {
            let category = category.to_lowercase();
            category == "monteur" || category == "test"
        })
        .unwrap_or(false)
}

pub(super) fn sort_contacts(
    mut contacts: Vec<PlanningContactRecord>,
) -> Vec<PlanningContactRecord> {
    contacts.sort_by(|left_contact, right_contact| {
        contact_display_name(left_contact)
            .to_lowercase()
            .cmp(&contact_display_name(right_contact).to_lowercase())
    });
    contacts
}

pub(super) fn contact_display_name(contact: &PlanningContactRecord) -> String {
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
                label: normalize_string_option(url.label),
                url: normalize_string_option(url.url),
                note: normalize_string_option(url.note),
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

#[cfg(test)]
mod tests {
    use super::*;

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
            urls: vec![DayliteContactUrl {
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
    fn filters_planning_contacts_keeps_monteur_and_test_sorted_by_display_name() {
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
            PlanningContactRecord {
                reference: "/v1/contacts/3004".to_string(),
                full_name: Some("Bea Test".to_string()),
                nickname: None,
                category: Some("Test".to_string()),
                urls: vec![],
            },
        ];

        let mapped = sort_contacts(filter_planning_contacts(contacts));

        // Vertrieb is dropped; Monteur and Test are kept and sorted by display name.
        assert_eq!(mapped.len(), 3);
        assert_eq!(mapped[0].reference, "/v1/contacts/3004"); // Bea Test
        assert_eq!(mapped[1].reference, "/v1/contacts/3003"); // Maks
        assert_eq!(mapped[2].reference, "/v1/contacts/3001"); // Zora Monteur
    }
}
