use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Default, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalStore {
    pub api_endpoints: ApiEndpoints,
    pub employee_settings: Vec<EmployeeSetting>,
    #[serde(default)]
    pub display_settings: DisplaySettings,
    pub daylite_cache: DayliteCache,
    #[serde(default)]
    pub holiday_cache: Vec<HolidayCacheEntry>,
}

impl LocalStore {
    pub fn cleanup_holiday_cache(&mut self, today: NaiveDate) {
        let one_year_ago = today
            .checked_sub_months(chrono::Months::new(12))
            .unwrap_or(today);
        self.holiday_cache.retain(|entry| {
            NaiveDate::parse_from_str(&entry.fetched_at, "%Y-%m-%d")
                .map(|d| d > one_year_ago)
                .unwrap_or(true)
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiEndpoints {
    pub daylite_base_url: String,
    pub planradar_base_url: String,
    #[serde(default)]
    pub zep_caldav_root_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeSetting {
    pub daylite_contact_reference: String,
    /// Old `primaryIcalUrl` values are not migrated automatically.
    #[serde(default)]
    pub zep_primary_calendar: Option<String>,
    #[serde(default)]
    pub zep_absence_calendar: Option<String>,
    #[serde(default)]
    pub primary_ical_last_tested_at: Option<String>,
    #[serde(default)]
    pub primary_ical_last_test_passed: Option<bool>,
    #[serde(default)]
    pub absence_ical_last_tested_at: Option<String>,
    #[serde(default)]
    pub absence_ical_last_test_passed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DisplaySettings {
    pub hide_non_plannable_employees: bool,
    #[serde(default)]
    pub show_weekend: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            hide_non_plannable_employees: true,
            show_weekend: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteCache {
    pub last_synced_at: Option<String>,
    pub projects: Vec<DayliteProjectCacheEntry>,
    pub contacts: Vec<DayliteContactCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteProjectCacheEntry {
    pub reference: String,
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteContactCacheEntry {
    pub reference: String,
    pub full_name: Option<String>,
    pub nickname: Option<String>,
    pub category: Option<String>,
    pub urls: Vec<DayliteContactUrl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteContactUrl {
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HolidayCacheEntry {
    pub year: i32,
    pub fetched_at: String,
    pub holidays: Vec<CachedHoliday>,
}

// Kept separate from holidays::Holiday so the on-disk cache schema and the API
// surface can evolve independently.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CachedHoliday {
    pub date: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StoreError {
    pub code: StoreErrorCode,
    pub user_message: String,
    pub technical_message: String,
}

impl StoreError {
    pub(super) fn new(
        code: StoreErrorCode,
        user_message: impl Into<String>,
        technical_message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            user_message: user_message.into(),
            technical_message: technical_message.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StoreErrorCode {
    ReadFailed,
    WriteFailed,
    CorruptFile,
    MissingFields,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup_removes_entries_older_than_one_year() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let mut store = LocalStore {
            holiday_cache: vec![
                HolidayCacheEntry {
                    year: 2024,
                    fetched_at: "2024-07-01".to_string(),
                    holidays: vec![],
                },
                HolidayCacheEntry {
                    year: 2023,
                    fetched_at: "2024-06-29".to_string(),
                    holidays: vec![],
                },
            ],
            ..LocalStore::default()
        };

        store.cleanup_holiday_cache(today);

        assert_eq!(store.holiday_cache.len(), 1);
        assert_eq!(store.holiday_cache[0].year, 2024);
    }

    #[test]
    fn cleanup_keeps_entries_within_one_year() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let mut store = LocalStore {
            holiday_cache: vec![HolidayCacheEntry {
                year: 2025,
                fetched_at: "2025-06-01".to_string(),
                holidays: vec![],
            }],
            ..LocalStore::default()
        };

        store.cleanup_holiday_cache(today);

        assert_eq!(store.holiday_cache.len(), 1);
    }

    #[test]
    fn cleanup_removes_all_entries_when_all_expired() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let mut store = LocalStore {
            holiday_cache: vec![
                HolidayCacheEntry {
                    year: 2023,
                    fetched_at: "2024-06-29".to_string(),
                    holidays: vec![],
                },
                HolidayCacheEntry {
                    year: 2022,
                    fetched_at: "2023-01-01".to_string(),
                    holidays: vec![],
                },
            ],
            ..LocalStore::default()
        };

        store.cleanup_holiday_cache(today);

        assert!(store.holiday_cache.is_empty());
    }

    #[test]
    fn display_settings_default_hides_non_plannable_employees() {
        assert!(DisplaySettings::default().hide_non_plannable_employees);
        assert!(
            LocalStore::default()
                .display_settings
                .hide_non_plannable_employees
        );
    }

    #[test]
    fn display_settings_default_hides_weekend() {
        assert!(!DisplaySettings::default().show_weekend);
        assert!(!LocalStore::default().display_settings.show_weekend);
    }
}
