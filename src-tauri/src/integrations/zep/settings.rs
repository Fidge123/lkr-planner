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

/// Takes results from a copy tested while the store could change, only for calendars still configured and untested.
pub(crate) fn adopt_calendar_tests(settings: &mut [EmployeeSetting], tested: &[EmployeeSetting]) {
    for setting in settings.iter_mut() {
        let Some(test) = tested
            .iter()
            .find(|t| t.daylite_contact_reference == setting.daylite_contact_reference)
        else {
            continue;
        };
        if setting.primary_ical_last_tested_at.is_none()
            && setting.zep_primary_calendar == test.zep_primary_calendar
        {
            setting.primary_ical_last_tested_at = test.primary_ical_last_tested_at.clone();
            setting.primary_ical_last_test_passed = test.primary_ical_last_test_passed;
        }
        if setting.absence_ical_last_tested_at.is_none()
            && setting.zep_absence_calendar == test.zep_absence_calendar
        {
            setting.absence_ical_last_tested_at = test.absence_ical_last_tested_at.clone();
            setting.absence_ical_last_test_passed = test.absence_ical_last_test_passed;
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

    fn primary(url: &str, tested_at: Option<&str>) -> EmployeeSetting {
        EmployeeSetting {
            daylite_contact_reference: "/v1/contacts/4".to_string(),
            zep_primary_calendar: Some(url.to_string()),
            primary_ical_last_tested_at: tested_at.map(ToString::to_string),
            primary_ical_last_test_passed: tested_at.map(|_| true),
            ..Default::default()
        }
    }

    #[test]
    fn adopts_the_test_of_a_calendar_that_is_still_configured() {
        let mut settings = vec![primary("https://zep/a/", None)];

        adopt_calendar_tests(&mut settings, &[primary("https://zep/a/", Some("t1"))]);

        assert_eq!(settings, vec![primary("https://zep/a/", Some("t1"))]);
    }

    #[test]
    fn leaves_a_calendar_saved_during_the_test_untested() {
        let mut settings = vec![primary("https://zep/b/", None)];

        adopt_calendar_tests(&mut settings, &[primary("https://zep/a/", Some("t1"))]);

        assert_eq!(settings, vec![primary("https://zep/b/", None)]);
    }

    #[test]
    fn keeps_a_result_stored_during_the_test() {
        let mut settings = vec![primary("https://zep/a/", Some("t2"))];

        adopt_calendar_tests(&mut settings, &[primary("https://zep/a/", Some("t1"))]);

        assert_eq!(settings, vec![primary("https://zep/a/", Some("t2"))]);
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
