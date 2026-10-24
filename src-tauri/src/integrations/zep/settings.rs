use super::caldav::probe_calendar;
use super::credentials::load_zep_credentials;
use super::types::ZepCalendarTestResult;
use crate::error::AppError;
use crate::integrations::local_store::EmployeeSetting;
use chrono::{SecondsFormat, Utc};

pub(crate) fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Skips silently without ZEP credentials; runs on startup paths that must not error.
pub(crate) async fn test_untested_calendar_urls(settings: &mut [EmployeeSetting]) {
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

    let creds = match load_zep_credentials() {
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

pub(crate) async fn run_calendar_test(
    calendar_url: &str,
) -> Result<ZepCalendarTestResult, AppError> {
    let creds = load_zep_credentials()?;
    let timestamp = current_timestamp();
    let (success, error_message) =
        match probe_calendar(calendar_url, &creds.username, &creds.password).await {
            Ok(()) => (true, None),
            Err(e) => (false, Some(e.user_message.clone())),
        };

    Ok(ZepCalendarTestResult {
        success,
        timestamp,
        error_message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_untested_does_nothing_when_all_urls_already_tested() {
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
    }

    #[tokio::test]
    async fn test_untested_skips_gracefully_when_zep_credentials_missing() {
        // No ZEP credentials in the test keychain; must return before any network call or timestamp update.
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/2".to_string(),
            zep_primary_calendar: Some("https://app.zep.de/caldav/admin/emp2/".to_string()),
            ..Default::default()
        }];

        test_untested_calendar_urls(&mut settings).await;

        assert_eq!(settings[0].primary_ical_last_tested_at, None);
        assert_eq!(settings[0].primary_ical_last_test_passed, None);
    }

    #[tokio::test]
    async fn test_untested_does_nothing_when_no_calendar_urls_set() {
        let mut settings = vec![EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/3".to_string(),
            ..Default::default()
        }];

        test_untested_calendar_urls(&mut settings).await;

        assert_eq!(settings[0].primary_ical_last_tested_at, None);
    }
}
