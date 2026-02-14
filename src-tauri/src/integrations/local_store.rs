use serde::{Deserialize, Serialize};
use specta::Type;
use std::fs;
use std::path::Path;
use tauri::Manager;

const STORE_FILE_NAME: &str = "local-store.json";

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalStore {
    pub api_endpoints: ApiEndpoints,
    pub token_references: TokenReferences,
    pub employee_settings: Vec<EmployeeSetting>,
    pub project_proposal_filters: ProjectProposalFilters,
    pub contact_filter: ContactFilter,
    pub routing_settings: RoutingSettings,
    #[serde(default)]
    pub daylite_cache: DayliteCache,
}

impl Default for LocalStore {
    fn default() -> Self {
        Self {
            api_endpoints: ApiEndpoints::default(),
            token_references: TokenReferences::default(),
            employee_settings: Vec::new(),
            project_proposal_filters: ProjectProposalFilters::default(),
            contact_filter: ContactFilter::default(),
            routing_settings: RoutingSettings::default(),
            daylite_cache: DayliteCache::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ApiEndpoints {
    pub daylite_base_url: String,
    pub planradar_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenReferences {
    pub daylite_token_reference: String,
    pub planradar_token_reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeSetting {
    pub employee_id: String,
    pub daylite_contact_reference: String,
    pub primary_ical_url: String,
    pub absence_ical_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectProposalFilters {
    pub pipelines: Vec<String>,
    pub columns: Vec<String>,
    pub categories: Vec<String>,
    pub exclusion_statuses: Vec<String>,
}

impl Default for ProjectProposalFilters {
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
    pub display_name: String,
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
    let store_path = app
        .path()
        .app_config_dir()
        .map(|path| path.join(STORE_FILE_NAME))
        .map_err(|error| StoreError {
            code: StoreErrorCode::WriteFailed,
            user_message: "Die lokale Konfiguration konnte nicht gespeichert werden.".to_string(),
            technical_message: format!("Pfad konnte nicht aufgelöst werden: {error}"),
        })?;

    save_store_to_path(&store_path, &store)
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
            },
            token_references: TokenReferences {
                daylite_token_reference: "keychain://daylite-token".to_string(),
                planradar_token_reference: "keychain://planradar-token".to_string(),
            },
            employee_settings: vec![EmployeeSetting {
                employee_id: "emp-1".to_string(),
                daylite_contact_reference: "/v1/contacts/100".to_string(),
                primary_ical_url: "https://example.com/primary.ics".to_string(),
                absence_ical_url: "https://example.com/absence.ics".to_string(),
            }],
            project_proposal_filters: ProjectProposalFilters {
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
                    display_name: "Max Mustermann".to_string(),
                }],
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
}
