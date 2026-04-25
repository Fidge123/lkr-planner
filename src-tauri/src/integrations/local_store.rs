use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

const STORE_FILE_NAME: &str = "local-store.json";

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalStore {
    pub api_endpoints: ApiEndpoints,
    pub token_references: TokenReferences,
    pub employee_settings: Vec<EmployeeSetting>,
    pub standard_filter: StandardFilter,
    pub contact_filter: ContactFilter,
    pub routing_settings: RoutingSettings,
    pub daylite_cache: DayliteCache,
    #[serde(default)]
    pub holiday_cache: Vec<HolidayCacheEntry>,
}

impl Default for LocalStore {
    fn default() -> Self {
        Self {
            api_endpoints: ApiEndpoints::default(),
            token_references: TokenReferences::default(),
            employee_settings: Vec::new(),
            standard_filter: StandardFilter::default(),
            contact_filter: ContactFilter::default(),
            routing_settings: RoutingSettings::default(),
            daylite_cache: DayliteCache::default(),
            holiday_cache: Vec::new(),
        }
    }
}

impl LocalStore {
    pub fn cleanup_holiday_cache(&mut self, today: NaiveDate) {
        let one_year_ago = today - chrono::Duration::days(365);
        self.holiday_cache.retain(|entry| {
            NaiveDate::parse_from_str(&entry.fetched_at, "%Y-%m-%d")
                .map(|d| d > one_year_ago)
                .unwrap_or(false)
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
pub struct TokenReferences {
    pub daylite_token_reference: String,
    pub planradar_token_reference: String,
    pub daylite_access_token: String,
    pub daylite_refresh_token: String,
    #[specta(type = Option<f64>)]
    pub daylite_access_token_expires_at_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeSetting {
    pub daylite_contact_reference: String,
    /// Full CalDAV URL of the primary (Einsatz) calendar, discovered via PROPFIND.
    /// None = no calendar assigned. Old `primaryIcalUrl` values are not migrated automatically.
    #[serde(default)]
    pub zep_primary_calendar: Option<String>,
    /// Full CalDAV URL of the absence (Abwesenheit) calendar, discovered via PROPFIND.
    /// None = no absence calendar (intentional, not an error).
    #[serde(default)]
    pub zep_absence_calendar: Option<String>,
    /// ISO 8601 timestamp of the last connection test for the primary calendar.
    /// None = never tested (or URL changed since last test).
    #[serde(default)]
    pub primary_ical_last_tested_at: Option<String>,
    /// Whether the last connection test for the primary calendar succeeded.
    /// None if never tested or URL changed since last test.
    #[serde(default)]
    pub primary_ical_last_test_passed: Option<bool>,
    /// ISO 8601 timestamp of the last connection test for the absence calendar.
    /// None = never tested (or URL changed since last test).
    #[serde(default)]
    pub absence_ical_last_tested_at: Option<String>,
    /// Whether the last connection test for the absence calendar succeeded.
    /// None if never tested or URL changed since last test.
    #[serde(default)]
    pub absence_ical_last_test_passed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StandardFilter {
    pub pipelines: Vec<String>,
    pub columns: Vec<String>,
    pub categories: Vec<String>,
    pub exclusion_statuses: Vec<String>,
}

impl Default for StandardFilter {
    fn default() -> Self {
        Self {
            pipelines: vec!["Aufträge".to_string()],
            columns: vec!["Vorbereitung".to_string(), "Durchführung".to_string()],
            categories: vec!["Überfällig".to_string(), "Liefertermin bekannt".to_string()],
            exclusion_statuses: vec!["Done".to_string()],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContactFilter {
    pub active_employee_keyword: String,
}

impl Default for ContactFilter {
    fn default() -> Self {
        Self {
            active_employee_keyword: "Monteur".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoutingSettings {
    pub openrouteservice_api_key: String,
    pub openrouteservice_profile: String,
}

impl Default for RoutingSettings {
    fn default() -> Self {
        Self {
            openrouteservice_api_key: String::new(),
            openrouteservice_profile: "driving-car".to_string(),
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
    pub urls: Vec<DayliteContactUrlCacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DayliteContactUrlCacheEntry {
    pub label: Option<String>,
    pub url: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct HolidayCacheEntry {
    pub year: i32,
    pub fetched_at: String,
    pub holidays: Vec<CachedHoliday>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StoreErrorCode {
    ReadFailed,
    WriteFailed,
    CorruptFile,
    MissingFields,
}

#[tauri::command]
#[specta::specta]
pub fn load_local_store(app: tauri::AppHandle) -> Result<LocalStore, StoreError> {
    let store_path = app
        .path()
        .app_config_dir()
        .map(|path| path.join(STORE_FILE_NAME))
        .map_err(|error| StoreError {
            code: StoreErrorCode::ReadFailed,
            user_message: "Die lokale Konfiguration konnte nicht geladen werden.".to_string(),
            technical_message: format!("Pfad konnte nicht aufgelöst werden: {error}"),
        })?;

    load_store_from_path(&store_path)
}

#[tauri::command]
#[specta::specta]
pub fn save_local_store(app: tauri::AppHandle, store: LocalStore) -> Result<(), StoreError> {
    save_store_internal(&app, store)
}

pub(crate) fn save_store_internal(
    app: &tauri::AppHandle,
    mut store: LocalStore,
) -> Result<(), StoreError> {
    let store_path = get_store_path(app)?;
    store.cleanup_holiday_cache(chrono::Local::now().date_naive());
    save_store_to_path(&store_path, &store)
}

fn get_store_path(app: &tauri::AppHandle) -> Result<PathBuf, StoreError> {
    app.path()
        .app_config_dir()
        .map(|path| path.join(STORE_FILE_NAME))
        .map_err(|error| StoreError {
            code: StoreErrorCode::WriteFailed,
            user_message: "Die lokale Konfiguration konnte nicht gespeichert werden.".to_string(),
            technical_message: format!("Pfad konnte nicht aufgelöst werden: {error}"),
        })
}

fn load_store_from_path(path: &Path) -> Result<LocalStore, StoreError> {
    if !path.exists() {
        return Ok(LocalStore::default());
    }

    let content = fs::read_to_string(path).map_err(|error| StoreError {
        code: StoreErrorCode::ReadFailed,
        user_message: "Die lokale Konfiguration konnte nicht geladen werden.".to_string(),
        technical_message: format!(
            "Datei konnte nicht gelesen werden ({}): {error}",
            path.display()
        ),
    })?;

    serde_json::from_str::<LocalStore>(&content).map_err(|error| {
        if error.to_string().contains("missing field") {
            return StoreError {
                code: StoreErrorCode::MissingFields,
                user_message: "Die lokale Konfiguration ist unvollständig und muss geprüft werden."
                    .to_string(),
                technical_message: format!("Fehlende Felder in {}: {error}", path.display()),
            };
        }

        StoreError {
            code: StoreErrorCode::CorruptFile,
            user_message:
                "Die lokale Konfiguration ist beschädigt und konnte nicht gelesen werden."
                    .to_string(),
            technical_message: format!("Ungültiges JSON in {}: {error}", path.display()),
        }
    })
}

fn save_store_to_path(path: &Path, store: &LocalStore) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| StoreError {
            code: StoreErrorCode::WriteFailed,
            user_message: "Die lokale Konfiguration konnte nicht gespeichert werden.".to_string(),
            technical_message: format!(
                "Verzeichnis konnte nicht erstellt werden ({}): {error}",
                parent.display()
            ),
        })?;
    }

    let serialized_store = serde_json::to_string_pretty(store).map_err(|error| StoreError {
        code: StoreErrorCode::WriteFailed,
        user_message: "Die lokale Konfiguration konnte nicht gespeichert werden.".to_string(),
        technical_message: format!("Serialisierung fehlgeschlagen: {error}"),
    })?;

    fs::write(path, serialized_store).map_err(|error| StoreError {
        code: StoreErrorCode::WriteFailed,
        user_message: "Die lokale Konfiguration konnte nicht gespeichert werden.".to_string(),
        technical_message: format!(
            "Datei konnte nicht geschrieben werden ({}): {error}",
            path.display()
        ),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn loads_defaults_for_missing_store_file() {
        let test_path = unique_test_path("missing-local-store.json");

        let loaded_store = load_store_from_path(&test_path).expect("load should succeed");

        assert_eq!(loaded_store, LocalStore::default());
    }

    #[test]
    fn saves_and_loads_store_restart_safe() {
        let test_path = unique_test_path("local-store.json");
        let store = LocalStore {
            api_endpoints: ApiEndpoints {
                daylite_base_url: "https://daylite.example/v1".to_string(),
                planradar_base_url: "https://planradar.example/api".to_string(),
                zep_caldav_root_url: "https://app.zep.de/caldav/admin".to_string(),
            },
            token_references: TokenReferences {
                daylite_token_reference: "keychain://daylite-token".to_string(),
                planradar_token_reference: "keychain://planradar-token".to_string(),
                daylite_access_token: "access-token-1".to_string(),
                daylite_refresh_token: "refresh-token-1".to_string(),
                daylite_access_token_expires_at_ms: Some(1_761_200_000_000),
            },
            employee_settings: vec![EmployeeSetting {
                daylite_contact_reference: "/v1/contacts/100".to_string(),
                zep_primary_calendar: Some(
                    "https://app.zep.de/caldav/admin/emp-1-primary/".to_string(),
                ),
                zep_absence_calendar: None,
                primary_ical_last_tested_at: Some("2026-03-01T10:00:00Z".to_string()),
                primary_ical_last_test_passed: Some(true),
                absence_ical_last_tested_at: None,
                absence_ical_last_test_passed: None,
            }],
            standard_filter: StandardFilter {
                pipelines: vec!["Aufträge".to_string()],
                columns: vec!["Vorbereitung".to_string()],
                categories: vec!["Überfällig".to_string()],
                exclusion_statuses: vec!["Done".to_string()],
            },
            contact_filter: ContactFilter {
                active_employee_keyword: "Monteur".to_string(),
            },
            routing_settings: RoutingSettings {
                openrouteservice_api_key: "ors-key".to_string(),
                openrouteservice_profile: "driving-car".to_string(),
            },
            daylite_cache: DayliteCache {
                last_synced_at: Some("2026-02-13T12:00:00Z".to_string()),
                projects: vec![DayliteProjectCacheEntry {
                    reference: "/v1/projects/1".to_string(),
                    name: "Projekt Nord".to_string(),
                    status: "in_progress".to_string(),
                }],
                contacts: vec![DayliteContactCacheEntry {
                    reference: "/v1/contacts/1".to_string(),
                    full_name: Some("Max Mustermann".to_string()),
                    nickname: Some("Max".to_string()),
                    category: Some("Monteur".to_string()),
                    urls: vec![DayliteContactUrlCacheEntry {
                        label: Some("Einsatz iCal".to_string()),
                        url: Some("https://example.com/primary.ics".to_string()),
                        note: None,
                    }],
                }],
            },
            holiday_cache: vec![HolidayCacheEntry {
                year: 2026,
                fetched_at: "2026-02-01".to_string(),
                holidays: vec![CachedHoliday {
                    date: "2026-01-01".to_string(),
                    name: "Neujahr".to_string(),
                }],
            }],
        };

        save_store_to_path(&test_path, &store).expect("save should succeed");
        let loaded_store = load_store_from_path(&test_path).expect("reload should succeed");

        assert_eq!(loaded_store, store);
    }

    #[test]
    fn returns_german_error_with_technical_details_for_corrupt_json() {
        let test_path = unique_test_path("corrupt-store.json");
        write_test_file(&test_path, "{invalid json");

        let error = load_store_from_path(&test_path).expect_err("load should fail");

        assert_eq!(error.code, StoreErrorCode::CorruptFile);
        assert_eq!(
            error.user_message,
            "Die lokale Konfiguration ist beschädigt und konnte nicht gelesen werden."
        );
        assert!(!error.technical_message.is_empty());
    }

    #[test]
    fn returns_german_error_with_technical_details_for_missing_fields() {
        let test_path = unique_test_path("missing-fields-store.json");
        write_test_file(
            &test_path,
            r#"{
              "apiEndpoints": {
                "dayliteBaseUrl": "https://daylite.example/v1"
              }
            }"#,
        );

        let error = load_store_from_path(&test_path).expect_err("load should fail");

        assert_eq!(error.code, StoreErrorCode::MissingFields);
        assert_eq!(
            error.user_message,
            "Die lokale Konfiguration ist unvollständig und muss geprüft werden."
        );
        assert!(!error.technical_message.is_empty());
    }

    fn unique_test_path(file_name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        path.push(format!("lkr-planner-local-store-tests-{now}"));
        path.push(file_name);
        path
    }

    fn write_test_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("directory should be creatable");
        }

        fs::write(path, content).expect("test file should be writable");
    }

    // Task 2.3: Cache age logic

    #[test]
    fn cleanup_removes_entries_older_than_one_year() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let mut store = LocalStore::default();
        store.holiday_cache = vec![
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
        ];

        store.cleanup_holiday_cache(today);

        assert_eq!(store.holiday_cache.len(), 1);
        assert_eq!(store.holiday_cache[0].year, 2024);
    }

    #[test]
    fn cleanup_keeps_entries_within_one_year() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let mut store = LocalStore::default();
        store.holiday_cache = vec![HolidayCacheEntry {
            year: 2025,
            fetched_at: "2025-06-01".to_string(),
            holidays: vec![],
        }];

        store.cleanup_holiday_cache(today);

        assert_eq!(store.holiday_cache.len(), 1);
    }

    #[test]
    fn cleanup_removes_all_entries_when_all_expired() {
        let today = NaiveDate::from_ymd_opt(2025, 6, 30).unwrap();
        let mut store = LocalStore::default();
        store.holiday_cache = vec![
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
        ];

        store.cleanup_holiday_cache(today);

        assert!(store.holiday_cache.is_empty());
    }

    #[test]
    fn store_with_holiday_cache_roundtrips_via_json() {
        let test_path = unique_test_path("holiday-cache-store.json");
        let mut store = LocalStore::default();
        store.holiday_cache = vec![HolidayCacheEntry {
            year: 2025,
            fetched_at: "2025-03-01".to_string(),
            holidays: vec![CachedHoliday {
                date: "2025-01-01".to_string(),
                name: "Neujahr".to_string(),
            }],
        }];

        save_store_to_path(&test_path, &store).expect("save should succeed");
        let loaded = load_store_from_path(&test_path).expect("reload should succeed");

        assert_eq!(loaded.holiday_cache.len(), 1);
        assert_eq!(loaded.holiday_cache[0].year, 2025);
        assert_eq!(loaded.holiday_cache[0].holidays[0].name, "Neujahr");
    }

    #[test]
    fn store_without_holiday_cache_field_loads_with_empty_cache() {
        let test_path = unique_test_path("no-holiday-cache.json");
        write_test_file(
            &test_path,
            r#"{
              "apiEndpoints": {"dayliteBaseUrl":"","planradarBaseUrl":"","zepCaldavRootUrl":""},
              "tokenReferences": {"dayliteTokenReference":"","planradarTokenReference":"","dayliteAccessToken":"","dayliteRefreshToken":""},
              "employeeSettings": [],
              "standardFilter": {"pipelines":[],"columns":[],"categories":[],"exclusionStatuses":[]},
              "contactFilter": {"activeEmployeeKeyword":""},
              "routingSettings": {"openrouteserviceApiKey":"","openrouteserviceProfile":""},
              "dayliteCache": {"projects":[],"contacts":[]}
            }"#,
        );

        let loaded = load_store_from_path(&test_path).expect("should load without holidayCache");
        assert!(loaded.holiday_cache.is_empty());
    }
}
