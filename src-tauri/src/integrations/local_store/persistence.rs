use super::types::{LocalStore, StoreError, StoreErrorCode};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

const STORE_FILE_NAME: &str = "local-store.json";

#[tauri::command]
#[specta::specta]
pub fn load_local_store(app: tauri::AppHandle) -> Result<LocalStore, StoreError> {
    let store_path = app
        .path()
        .app_config_dir()
        .map(|path| path.join(STORE_FILE_NAME))
        .map_err(|error| {
            StoreError::new(
                StoreErrorCode::ReadFailed,
                "Die lokale Konfiguration konnte nicht geladen werden.",
                format!("Pfad konnte nicht aufgelöst werden: {error}"),
            )
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
    // Cleans up expired holiday cache entries as a side-effect of every save.
    let store_path = get_store_path(app)?;
    store.cleanup_holiday_cache(chrono::Utc::now().date_naive());
    save_store_to_path(&store_path, &store)
}

fn get_store_path(app: &tauri::AppHandle) -> Result<PathBuf, StoreError> {
    app.path()
        .app_config_dir()
        .map(|path| path.join(STORE_FILE_NAME))
        .map_err(|error| {
            StoreError::new(
                StoreErrorCode::WriteFailed,
                "Die lokale Konfiguration konnte nicht gespeichert werden.",
                format!("Pfad konnte nicht aufgelöst werden: {error}"),
            )
        })
}

fn load_store_from_path(path: &Path) -> Result<LocalStore, StoreError> {
    if !path.exists() {
        return Ok(LocalStore::default());
    }

    let content = fs::read_to_string(path).map_err(|error| {
        StoreError::new(
            StoreErrorCode::ReadFailed,
            "Die lokale Konfiguration konnte nicht geladen werden.",
            format!(
                "Datei konnte nicht gelesen werden ({}): {error}",
                path.display()
            ),
        )
    })?;

    serde_json::from_str::<LocalStore>(&content).map_err(|error| {
        if error.to_string().contains("missing field") {
            return StoreError::new(
                StoreErrorCode::MissingFields,
                "Die lokale Konfiguration ist unvollständig und muss geprüft werden.",
                format!("Fehlende Felder in {}: {error}", path.display()),
            );
        }

        StoreError::new(
            StoreErrorCode::CorruptFile,
            "Die lokale Konfiguration ist beschädigt und konnte nicht gelesen werden.",
            format!("Ungültiges JSON in {}: {error}", path.display()),
        )
    })
}

fn save_store_to_path(path: &Path, store: &LocalStore) -> Result<(), StoreError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            StoreError::new(
                StoreErrorCode::WriteFailed,
                "Die lokale Konfiguration konnte nicht gespeichert werden.",
                format!(
                    "Verzeichnis konnte nicht erstellt werden ({}): {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let serialized_store = serde_json::to_string_pretty(store).map_err(|error| {
        StoreError::new(
            StoreErrorCode::WriteFailed,
            "Die lokale Konfiguration konnte nicht gespeichert werden.",
            format!("Serialisierung fehlgeschlagen: {error}"),
        )
    })?;

    fs::write(path, serialized_store).map_err(|error| {
        StoreError::new(
            StoreErrorCode::WriteFailed,
            "Die lokale Konfiguration konnte nicht gespeichert werden.",
            format!(
                "Datei konnte nicht geschrieben werden ({}): {error}",
                path.display()
            ),
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::local_store::types::{
        ApiEndpoints, CachedHoliday, DayliteCache, DayliteContactCacheEntry, DayliteContactUrl,
        DayliteProjectCacheEntry, DisplaySettings, EmployeeSetting, HolidayCacheEntry,
    };
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
            display_settings: DisplaySettings {
                hide_non_plannable_employees: false,
                show_weekend: false,
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
                    urls: vec![DayliteContactUrl {
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

    #[test]
    fn store_with_holiday_cache_roundtrips_via_json() {
        let test_path = unique_test_path("holiday-cache-store.json");
        let store = LocalStore {
            holiday_cache: vec![HolidayCacheEntry {
                year: 2025,
                fetched_at: "2025-03-01".to_string(),
                holidays: vec![CachedHoliday {
                    date: "2025-01-01".to_string(),
                    name: "Neujahr".to_string(),
                }],
            }],
            ..LocalStore::default()
        };

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
              "employeeSettings": [],
              "dayliteCache": {"projects":[],"contacts":[]}
            }"#,
        );

        let loaded = load_store_from_path(&test_path).expect("should load without holidayCache");
        assert!(loaded.holiday_cache.is_empty());
        // displaySettings absent in the file must default to hiding non-plannable employees.
        assert!(loaded.display_settings.hide_non_plannable_employees);
    }

    #[test]
    fn show_weekend_round_trips_through_save_and_load() {
        let test_path = unique_test_path("show-weekend-round-trip.json");
        let store = LocalStore {
            display_settings: DisplaySettings {
                hide_non_plannable_employees: true,
                show_weekend: true,
            },
            ..LocalStore::default()
        };

        save_store_to_path(&test_path, &store).expect("save should succeed");
        let loaded = load_store_from_path(&test_path).expect("reload should succeed");

        assert!(loaded.display_settings.show_weekend);
        assert!(loaded.display_settings.hide_non_plannable_employees);
    }

    #[test]
    fn display_settings_without_show_weekend_field_defaults_to_false() {
        let test_path = unique_test_path("no-show-weekend.json");
        write_test_file(
            &test_path,
            r#"{
              "apiEndpoints": {"dayliteBaseUrl":"","planradarBaseUrl":"","zepCaldavRootUrl":""},
              "employeeSettings": [],
              "displaySettings": {"hideNonPlannableEmployees": true},
              "dayliteCache": {"projects":[],"contacts":[]}
            }"#,
        );

        let loaded = load_store_from_path(&test_path).expect("should load without showWeekend");
        // showWeekend absent in a previously stored displaySettings must default to off.
        assert!(!loaded.display_settings.show_weekend);
        assert!(loaded.display_settings.hide_non_plannable_employees);
    }
}
