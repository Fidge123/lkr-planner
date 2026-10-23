use super::types::LocalStore;
use crate::error::{AppError, AppErrorCode};
use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::record_failure;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use tauri::Manager;

const STORE_FILE_NAME: &str = "local-store.json";

static STORE: StoreCell = StoreCell::new();

#[tauri::command]
#[specta::specta]
pub fn load_local_store(app: tauri::AppHandle) -> Result<LocalStore, AppError> {
    store_path(&app)
        .and_then(|path| STORE.read(&path))
        .inspect_err(|error| {
            record_failure(
                &app,
                Operation::LoadLocalStore,
                Integration::LocalStore,
                error,
            )
        })
}

#[tauri::command]
#[specta::specta]
pub fn save_local_store(app: tauri::AppHandle, store: LocalStore) -> Result<(), AppError> {
    update_local_store(&app, |current| *current = store)
}

/// Writers must change the store only through this, so concurrent commands cannot drop each other's changes.
pub(crate) fn update_local_store<R>(
    app: &tauri::AppHandle,
    change: impl FnOnce(&mut LocalStore) -> R,
) -> Result<R, AppError> {
    store_path(app)
        .and_then(|path| STORE.update(&path, change))
        .inspect_err(|error| {
            record_failure(
                app,
                Operation::SaveLocalStore,
                Integration::LocalStore,
                error,
            )
        })
}

/// The file is read once; afterwards memory is the source and every update is written through.
struct StoreCell(Mutex<Option<LocalStore>>);

impl StoreCell {
    const fn new() -> Self {
        Self(Mutex::new(None))
    }

    fn read(&self, path: &Path) -> Result<LocalStore, AppError> {
        let mut cached = self.lock();
        if let Some(store) = cached.as_ref() {
            return Ok(store.clone());
        }
        let store = load_store_from_path(path)?;
        *cached = Some(store.clone());
        Ok(store)
    }

    fn update<R>(
        &self,
        path: &Path,
        change: impl FnOnce(&mut LocalStore) -> R,
    ) -> Result<R, AppError> {
        let mut cached = self.lock();
        let mut store = match cached.as_ref() {
            Some(store) => store.clone(),
            None => load_store_from_path(path)?,
        };
        let result = change(&mut store);
        store.cleanup_holiday_cache(chrono::Utc::now().date_naive());
        save_store_to_path(path, &store)?;
        *cached = Some(store);
        Ok(result)
    }

    fn lock(&self) -> MutexGuard<'_, Option<LocalStore>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn store_path(app: &tauri::AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_config_dir()
        .map(|path| path.join(STORE_FILE_NAME))
        .map_err(|error| {
            AppError::new(
                AppErrorCode::ReadFailed,
                "Die lokale Konfiguration konnte nicht geladen werden.",
                format!("Pfad konnte nicht aufgelöst werden: {error}"),
            )
        })
}

fn load_store_from_path(path: &Path) -> Result<LocalStore, AppError> {
    if !path.exists() {
        return Ok(LocalStore::default());
    }

    let content = fs::read_to_string(path).map_err(|error| {
        AppError::new(
            AppErrorCode::ReadFailed,
            "Die lokale Konfiguration konnte nicht geladen werden.",
            format!(
                "Datei konnte nicht gelesen werden ({}): {error}",
                path.display()
            ),
        )
    })?;

    serde_json::from_str::<LocalStore>(&content).map_err(|error| {
        if error.to_string().contains("missing field") {
            return AppError::new(
                AppErrorCode::MissingFields,
                "Die lokale Konfiguration ist unvollständig und muss geprüft werden.",
                format!("Fehlende Felder in {}: {error}", path.display()),
            );
        }

        AppError::new(
            AppErrorCode::CorruptFile,
            "Die lokale Konfiguration ist beschädigt und konnte nicht gelesen werden.",
            format!("Ungültiges JSON in {}: {error}", path.display()),
        )
    })
}

fn save_store_to_path(path: &Path, store: &LocalStore) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::new(
                AppErrorCode::WriteFailed,
                "Die lokale Konfiguration konnte nicht gespeichert werden.",
                format!(
                    "Verzeichnis konnte nicht erstellt werden ({}): {error}",
                    parent.display()
                ),
            )
        })?;
    }

    let serialized_store = serde_json::to_string_pretty(store).map_err(|error| {
        AppError::new(
            AppErrorCode::WriteFailed,
            "Die lokale Konfiguration konnte nicht gespeichert werden.",
            format!("Serialisierung fehlgeschlagen: {error}"),
        )
    })?;

    fs::write(path, serialized_store).map_err(|error| {
        AppError::new(
            AppErrorCode::WriteFailed,
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
        DisplaySettings, EmployeeSetting, HolidayCacheEntry, TelemetrySettings,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn an_update_keeps_the_changes_of_earlier_updates() {
        let cell = StoreCell::new();
        let path = unique_test_path("updated-store.json");

        cell.update(&path, |store| store.display_settings.show_weekend = true)
            .expect("first update should succeed");
        cell.update(&path, |store| {
            store.api_endpoints.zep_caldav_root_url = "https://zep.example/caldav".to_string()
        })
        .expect("second update should succeed");

        let saved = load_store_from_path(&path).expect("saved store should load");
        assert!(saved.display_settings.show_weekend);
        assert_eq!(
            saved.api_endpoints.zep_caldav_root_url,
            "https://zep.example/caldav"
        );
    }

    #[test]
    fn reads_after_the_first_come_from_memory() {
        let cell = StoreCell::new();
        let path = unique_test_path("cached-store.json");
        cell.read(&path).expect("first read should succeed");

        let mut changed = LocalStore::default();
        changed.display_settings.show_weekend = true;
        save_store_to_path(&path, &changed).expect("save should succeed");

        assert_eq!(
            cell.read(&path).expect("cached read"),
            LocalStore::default()
        );
    }

    #[test]
    fn a_failed_write_keeps_the_previous_store() {
        let cell = StoreCell::new();
        let path = unique_test_path("kept-store.json");
        cell.update(&path, |store| store.display_settings.show_weekend = true)
            .expect("update should succeed");
        let blocker = unique_test_path("blocker");
        write_test_file(&blocker, "");

        let failed = cell.update(&blocker.join("store.json"), |store| {
            store.display_settings.show_weekend = false
        });

        assert!(failed.is_err());
        assert!(
            cell.read(&path)
                .expect("cached read")
                .display_settings
                .show_weekend
        );
    }

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
                primary_ical_last_tested_at: Some("2026-03-01T10:00:00Z".to_string()),
                primary_ical_last_test_passed: Some(true),
                ..Default::default()
            }],
            display_settings: DisplaySettings {
                hide_non_plannable_employees: false,
                show_weekend: false,
            },
            daylite_cache: DayliteCache {
                last_synced_at: Some("2026-02-13T12:00:00Z".to_string()),
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
            telemetry: TelemetrySettings {
                enabled: true,
                install_id: Some("11111111-2222-4333-8444-555555555555".to_string()),
            },
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

        assert_eq!(error.code, AppErrorCode::CorruptFile);
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

        assert_eq!(error.code, AppErrorCode::MissingFields);
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
    fn store_without_telemetry_field_loads_with_telemetry_disabled() {
        let test_path = unique_test_path("no-telemetry-section.json");
        write_test_file(
            &test_path,
            r#"{
              "apiEndpoints": {"dayliteBaseUrl":"","planradarBaseUrl":"","zepCaldavRootUrl":""},
              "employeeSettings": [],
              "dayliteCache": {"projects":[],"contacts":[]}
            }"#,
        );

        let loaded = load_store_from_path(&test_path).expect("should load without telemetry");

        assert!(!loaded.telemetry.enabled);
        assert_eq!(loaded.telemetry.install_id, None);
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
        assert!(!loaded.display_settings.show_weekend);
        assert!(loaded.display_settings.hide_non_plannable_employees);
    }
}
