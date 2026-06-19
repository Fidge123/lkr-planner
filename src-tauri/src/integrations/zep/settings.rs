use super::caldav::probe_calendar;
use super::credentials::load_zep_credentials_from_keychain;
use crate::integrations::local_store::EmployeeSetting;
use chrono::{SecondsFormat, Utc};

pub(super) fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Tests every calendar URL in `settings` that has no recorded test result yet
/// (i.e. `last_tested_at` is `None`). Silently does nothing if ZEP credentials
/// are not configured. Intended to be called after Daylite reconciliation so that
/// newly-discovered calendar URLs are validated immediately without a manual save.
pub(crate) async fn test_untested_calendar_urls(settings: &mut [EmployeeSetting]) {
    // Collect (contact_reference, url, is_primary) triples that need testing.
    let to_test: Vec<(String, String, bool)> = settings
        .iter()
        .flat_map(|s| {
            let mut items = vec![];
            if s.primary_ical_last_tested_at.is_none() {
                if let Some(ref url) = s.zep_primary_calendar {
                    items.push((s.daylite_contact_reference.clone(), url.clone(), true));
                }
            }
            if s.absence_ical_last_tested_at.is_none() {
                if let Some(ref url) = s.zep_absence_calendar {
                    items.push((s.daylite_contact_reference.clone(), url.clone(), false));
                }
            }
            items
        })
        .collect();

    if to_test.is_empty() {
        return;
    }

    let creds = match load_zep_credentials_from_keychain() {
        Ok(c) => c,
        Err(_) => return,
    };

    let timestamp = current_timestamp();
    for (reference, url, is_primary) in &to_test {
        let success = probe_calendar(url, &creds.username, &creds.password)
            .await
            .is_ok();
        if let Some(setting) = settings
            .iter_mut()
            .find(|s| s.daylite_contact_reference == *reference)
        {
            if *is_primary {
                setting.primary_ical_last_tested_at = Some(timestamp.clone());
                setting.primary_ical_last_test_passed = Some(success);
            } else {
                setting.absence_ical_last_tested_at = Some(timestamp.clone());
                setting.absence_ical_last_test_passed = Some(success);
            }
        }
    }
}

pub(super) fn find_or_default_setting(
    settings: &[EmployeeSetting],
    daylite_contact_reference: &str,
) -> EmployeeSetting {
    settings
        .iter()
        .find(|s| s.daylite_contact_reference == daylite_contact_reference)
        .cloned()
        .unwrap_or_else(|| EmployeeSetting {
            daylite_contact_reference: daylite_contact_reference.to_string(),
            ..Default::default()
        })
}

pub(super) fn update_setting(
    settings: &mut Vec<EmployeeSetting>,
    daylite_contact_reference: &str,
    update: impl FnOnce(&mut EmployeeSetting),
) {
    if let Some(setting) = settings
        .iter_mut()
        .find(|s| s.daylite_contact_reference == daylite_contact_reference)
    {
        update(setting);
    } else {
        let mut new_setting = EmployeeSetting {
            daylite_contact_reference: daylite_contact_reference.to_string(),
            ..Default::default()
        };
        update(&mut new_setting);
        settings.push(new_setting);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_untested_does_nothing_when_all_urls_already_tested() {
        tauri::async_runtime::block_on(async {
            let mut settings = vec![EmployeeSetting {
                daylite_contact_reference: "/v1/contacts/1".to_string(),
                zep_primary_calendar: Some("https://app.zep.de/caldav/admin/emp1/".to_string()),
                zep_absence_calendar: Some("https://app.zep.de/caldav/admin/emp1-ab/".to_string()),
                primary_ical_last_tested_at: Some("2026-01-01T12:00:00.000Z".to_string()),
                primary_ical_last_test_passed: Some(true),
                absence_ical_last_tested_at: Some("2026-01-01T12:00:00.000Z".to_string()),
                absence_ical_last_test_passed: Some(false),
            }];

            test_untested_calendar_urls(&mut settings).await;

            // Timestamps must not change because both are already tested.
            assert_eq!(
                settings[0].primary_ical_last_tested_at,
                Some("2026-01-01T12:00:00.000Z".to_string())
            );
            assert_eq!(settings[0].primary_ical_last_test_passed, Some(true));
            assert_eq!(
                settings[0].absence_ical_last_tested_at,
                Some("2026-01-01T12:00:00.000Z".to_string())
            );
            assert_eq!(settings[0].absence_ical_last_test_passed, Some(false));
        });
    }

    #[test]
    fn test_untested_skips_gracefully_when_zep_credentials_missing() {
        // In the test environment there are no ZEP credentials in the keychain,
        // so test_untested_calendar_urls must return without panicking or mutating
        // the test timestamps (credentials check happens before any network call).
        tauri::async_runtime::block_on(async {
            let mut settings = vec![EmployeeSetting {
                daylite_contact_reference: "/v1/contacts/2".to_string(),
                zep_primary_calendar: Some("https://app.zep.de/caldav/admin/emp2/".to_string()),
                zep_absence_calendar: None,
                primary_ical_last_tested_at: None,
                primary_ical_last_test_passed: None,
                absence_ical_last_tested_at: None,
                absence_ical_last_test_passed: None,
            }];

            test_untested_calendar_urls(&mut settings).await;

            // No credentials → timestamps stay None; no panic.
            assert_eq!(settings[0].primary_ical_last_tested_at, None);
            assert_eq!(settings[0].primary_ical_last_test_passed, None);
        });
    }

    #[test]
    fn test_untested_does_nothing_when_no_calendar_urls_set() {
        tauri::async_runtime::block_on(async {
            let mut settings = vec![EmployeeSetting {
                daylite_contact_reference: "/v1/contacts/3".to_string(),
                zep_primary_calendar: None,
                zep_absence_calendar: None,
                primary_ical_last_tested_at: None,
                primary_ical_last_test_passed: None,
                absence_ical_last_tested_at: None,
                absence_ical_last_test_passed: None,
            }];

            test_untested_calendar_urls(&mut settings).await;

            assert_eq!(settings[0].primary_ical_last_tested_at, None);
        });
    }

    #[test]
    fn update_setting_creates_new_entry_when_not_found() {
        let mut settings: Vec<EmployeeSetting> = vec![];
        update_setting(&mut settings, "/v1/contacts/42", |s| {
            s.zep_primary_calendar = Some("https://app.zep.de/cal/".to_string());
        });
        assert_eq!(settings.len(), 1);
        assert_eq!(
            settings[0].zep_primary_calendar,
            Some("https://app.zep.de/cal/".to_string())
        );
    }

    #[test]
    fn update_setting_updates_existing_entry_and_clears_timestamp() {
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/42".to_string(),
            zep_primary_calendar: None,
            zep_absence_calendar: None,
            primary_ical_last_tested_at: Some("2026-01-01T00:00:00Z".to_string()),
            primary_ical_last_test_passed: Some(true),
            absence_ical_last_tested_at: None,
            absence_ical_last_test_passed: None,
        }];
        update_setting(&mut settings, "/v1/contacts/42", |s| {
            s.zep_primary_calendar = Some("https://cal.example/".to_string());
            s.primary_ical_last_tested_at = None;
        });
        assert_eq!(settings.len(), 1);
        assert_eq!(
            settings[0].zep_primary_calendar,
            Some("https://cal.example/".to_string())
        );
        assert_eq!(settings[0].primary_ical_last_tested_at, None);
    }
}
