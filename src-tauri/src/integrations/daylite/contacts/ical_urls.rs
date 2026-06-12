use super::types::PlanningContactRecord;
use crate::integrations::local_store::{DayliteContactUrl, EmployeeSetting};

pub(super) fn merge_contact_ical_urls(
    existing_urls: Vec<DayliteContactUrl>,
    primary_ical_url: &str,
    absence_ical_url: &str,
) -> Vec<DayliteContactUrl> {
    let mut merged_urls = existing_urls
        .into_iter()
        .filter(|url| {
            let Some(label) = normalize_url_label(url.label.as_deref()) else {
                return true;
            };

            !is_primary_ical_label(&label) && !is_absence_ical_label(&label)
        })
        .collect::<Vec<_>>();

    if let Some(primary_url) = normalize_non_empty(primary_ical_url) {
        merged_urls.push(DayliteContactUrl {
            label: Some("Einsatz iCal".to_string()),
            url: Some(primary_url.to_string()),
            note: None,
        });
    }

    if let Some(absence_url) = normalize_non_empty(absence_ical_url) {
        merged_urls.push(DayliteContactUrl {
            label: Some("Abwesenheit iCal".to_string()),
            url: Some(absence_url.to_string()),
            note: None,
        });
    }

    merged_urls
}

/// Daylite is the source of truth for an employee's calendar configuration.
/// Whenever fresh contacts are fetched from Daylite, this mirrors the managed
/// "Einsatz iCal" / "Abwesenheit iCal" URLs from each contact into the local
/// employee settings, so a calendar configured on one device is picked up on
/// every other device. When a calendar URL changes (including being removed in
/// Daylite), the corresponding connection-test result is cleared because it no
/// longer describes the current URL.
pub(super) fn reconcile_employee_calendars_from_contacts(
    settings: &mut Vec<EmployeeSetting>,
    contacts: &[PlanningContactRecord],
) {
    for contact in contacts {
        let primary = extract_managed_ical_url(&contact.urls, is_primary_ical_label);
        let absence = extract_managed_ical_url(&contact.urls, is_absence_ical_label);

        if let Some(setting) = settings
            .iter_mut()
            .find(|setting| setting.daylite_contact_reference == contact.reference)
        {
            if setting.zep_primary_calendar != primary {
                setting.zep_primary_calendar = primary;
                setting.primary_ical_last_tested_at = None;
                setting.primary_ical_last_test_passed = None;
            }
            if setting.zep_absence_calendar != absence {
                setting.zep_absence_calendar = absence;
                setting.absence_ical_last_tested_at = None;
                setting.absence_ical_last_test_passed = None;
            }
        } else if primary.is_some() || absence.is_some() {
            settings.push(EmployeeSetting {
                daylite_contact_reference: contact.reference.clone(),
                zep_primary_calendar: primary,
                zep_absence_calendar: absence,
                ..Default::default()
            });
        }
    }
}

fn extract_managed_ical_url(
    urls: &[DayliteContactUrl],
    matches_label: fn(&str) -> bool,
) -> Option<String> {
    urls.iter().find_map(|url| {
        let label = normalize_url_label(url.label.as_deref())?;
        if !matches_label(&label) {
            return None;
        }

        url.url
            .as_deref()
            .and_then(normalize_non_empty)
            .map(ToString::to_string)
    })
}

fn is_primary_ical_label(label: &str) -> bool {
    label == "einsatz ical"
}

fn is_absence_ical_label(label: &str) -> bool {
    label == "abwesenheit ical"
}

fn normalize_url_label(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
}

fn normalize_non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_contact_ical_urls_keeps_unmanaged_labels() {
        let existing_urls = vec![
            DayliteContactUrl {
                label: Some("Website".to_string()),
                url: Some("https://example.com".to_string()),
                note: None,
            },
            DayliteContactUrl {
                label: Some("FR-Fehlzeiten".to_string()),
                url: Some("https://example.com/old-absence.ics".to_string()),
                note: None,
            },
            DayliteContactUrl {
                label: Some("Einsatz iCal".to_string()),
                url: Some("https://example.com/old-primary.ics".to_string()),
                note: None,
            },
        ];

        let merged = merge_contact_ical_urls(
            existing_urls,
            "https://example.com/new-primary.ics",
            "https://example.com/new-absence.ics",
        );

        assert_eq!(
            merged,
            vec![
                DayliteContactUrl {
                    label: Some("Website".to_string()),
                    url: Some("https://example.com".to_string()),
                    note: None,
                },
                DayliteContactUrl {
                    label: Some("FR-Fehlzeiten".to_string()),
                    url: Some("https://example.com/old-absence.ics".to_string()),
                    note: None,
                },
                DayliteContactUrl {
                    label: Some("Einsatz iCal".to_string()),
                    url: Some("https://example.com/new-primary.ics".to_string()),
                    note: None,
                },
                DayliteContactUrl {
                    label: Some("Abwesenheit iCal".to_string()),
                    url: Some("https://example.com/new-absence.ics".to_string()),
                    note: None,
                },
            ]
        );
    }

    #[test]
    fn reconcile_populates_calendars_from_daylite_for_new_device() {
        // Simulates a fresh device: no local employee settings exist yet, but the
        // contact fetched from Daylite carries the managed iCal URLs.
        let mut settings: Vec<EmployeeSetting> = vec![];
        let contacts = vec![PlanningContactRecord {
            reference: "/v1/contacts/100".to_string(),
            full_name: Some("Max Mustermann".to_string()),
            nickname: None,
            category: Some("Monteur".to_string()),
            urls: vec![
                DayliteContactUrl {
                    label: Some("Einsatz iCal".to_string()),
                    url: Some("https://example.com/max-primary.ics".to_string()),
                    note: None,
                },
                DayliteContactUrl {
                    label: Some("Abwesenheit iCal".to_string()),
                    url: Some("https://example.com/max-absence.ics".to_string()),
                    note: None,
                },
            ],
        }];

        reconcile_employee_calendars_from_contacts(&mut settings, &contacts);

        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].daylite_contact_reference, "/v1/contacts/100");
        assert_eq!(
            settings[0].zep_primary_calendar,
            Some("https://example.com/max-primary.ics".to_string())
        );
        assert_eq!(
            settings[0].zep_absence_calendar,
            Some("https://example.com/max-absence.ics".to_string())
        );
    }

    #[test]
    fn reconcile_overrides_changed_url_and_clears_stale_test_result() {
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/100".to_string(),
            zep_primary_calendar: Some("https://example.com/old-primary.ics".to_string()),
            zep_absence_calendar: None,
            primary_ical_last_tested_at: Some("2026-01-01T00:00:00Z".to_string()),
            primary_ical_last_test_passed: Some(true),
            absence_ical_last_tested_at: None,
            absence_ical_last_test_passed: None,
        }];
        let contacts = vec![PlanningContactRecord {
            reference: "/v1/contacts/100".to_string(),
            full_name: Some("Max Mustermann".to_string()),
            nickname: None,
            category: Some("Monteur".to_string()),
            urls: vec![DayliteContactUrl {
                label: Some("Einsatz iCal".to_string()),
                url: Some("https://example.com/new-primary.ics".to_string()),
                note: None,
            }],
        }];

        reconcile_employee_calendars_from_contacts(&mut settings, &contacts);

        assert_eq!(
            settings[0].zep_primary_calendar,
            Some("https://example.com/new-primary.ics".to_string())
        );
        // The recorded test no longer describes the new URL, so it is cleared.
        assert_eq!(settings[0].primary_ical_last_tested_at, None);
        assert_eq!(settings[0].primary_ical_last_test_passed, None);
    }

    #[test]
    fn reconcile_clears_calendar_removed_in_daylite() {
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/100".to_string(),
            zep_primary_calendar: Some("https://example.com/old-primary.ics".to_string()),
            zep_absence_calendar: Some("https://example.com/old-absence.ics".to_string()),
            primary_ical_last_tested_at: Some("2026-01-01T00:00:00Z".to_string()),
            primary_ical_last_test_passed: Some(true),
            absence_ical_last_tested_at: None,
            absence_ical_last_test_passed: None,
        }];
        let contacts = vec![PlanningContactRecord {
            reference: "/v1/contacts/100".to_string(),
            full_name: Some("Max Mustermann".to_string()),
            nickname: None,
            category: Some("Monteur".to_string()),
            urls: vec![],
        }];

        reconcile_employee_calendars_from_contacts(&mut settings, &contacts);

        assert_eq!(settings[0].zep_primary_calendar, None);
        assert_eq!(settings[0].zep_absence_calendar, None);
        assert_eq!(settings[0].primary_ical_last_tested_at, None);
    }

    #[test]
    fn reconcile_leaves_unchanged_calendar_and_test_result_intact() {
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/100".to_string(),
            zep_primary_calendar: Some("https://example.com/primary.ics".to_string()),
            zep_absence_calendar: None,
            primary_ical_last_tested_at: Some("2026-01-01T00:00:00Z".to_string()),
            primary_ical_last_test_passed: Some(true),
            absence_ical_last_tested_at: None,
            absence_ical_last_test_passed: None,
        }];
        let contacts = vec![PlanningContactRecord {
            reference: "/v1/contacts/100".to_string(),
            full_name: Some("Max Mustermann".to_string()),
            nickname: None,
            category: Some("Monteur".to_string()),
            urls: vec![DayliteContactUrl {
                label: Some("Einsatz iCal".to_string()),
                url: Some("https://example.com/primary.ics".to_string()),
                note: None,
            }],
        }];

        reconcile_employee_calendars_from_contacts(&mut settings, &contacts);

        // Unchanged URL must preserve the existing connection-test result.
        assert_eq!(
            settings[0].primary_ical_last_tested_at,
            Some("2026-01-01T00:00:00Z".to_string())
        );
        assert_eq!(settings[0].primary_ical_last_test_passed, Some(true));
    }

    #[test]
    fn reconcile_skips_contacts_without_managed_calendars() {
        let mut settings: Vec<EmployeeSetting> = vec![];
        let contacts = vec![PlanningContactRecord {
            reference: "/v1/contacts/100".to_string(),
            full_name: Some("Max Mustermann".to_string()),
            nickname: None,
            category: Some("Monteur".to_string()),
            urls: vec![DayliteContactUrl {
                label: Some("Website".to_string()),
                url: Some("https://example.com".to_string()),
                note: None,
            }],
        }];

        reconcile_employee_calendars_from_contacts(&mut settings, &contacts);

        assert!(settings.is_empty());
    }
}
