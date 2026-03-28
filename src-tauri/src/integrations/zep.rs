use crate::integrations::daylite::contacts::{
    sync_contact_ical_urls, DayliteUpdateContactIcalUrlsInput,
};
use crate::integrations::local_store::EmployeeSetting;
use crate::secret_manager;
use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::Method;

// ── Keychain identifiers ─────────────────────────────────────────────────────

const ZEP_KEYCHAIN_SERVICE: &str = "lkr-planner-zep";
const ZEP_KEYCHAIN_ACCOUNT: &str = "LKR Planner ZEP Admin";

// ── Public types ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepError {
    pub code: ZepErrorCode,
    pub user_message: String,
    pub technical_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ZepErrorCode {
    KeychainError,
    MissingCredentials,
    Unauthorized,
    NotFound,
    NetworkError,
    InvalidResponse,
    InvalidConfiguration,
    DayliteSyncFailed,
}

/// A calendar discovered via CalDAV PROPFIND.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCalendar {
    /// Human-readable display name from `<displayname>`.
    pub display_name: String,
    /// Full absolute URL of the calendar (used for connection tests and stored in EmployeeSetting).
    pub url: String,
}

/// Returned by the credential test command: number of calendars discovered.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCredentialTestResult {
    pub calendar_count: u32,
}

/// Publicly visible credential info (root URL + username, no password).
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCredentialsInfo {
    pub root_url: String,
    pub username: String,
}

/// Result of a combined save-and-test action for one calendar source.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ZepCalendarTestResult {
    pub success: bool,
    pub timestamp: String,
    pub error_message: Option<String>,
}

/// Which iCal source is being acted upon.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IcalSource {
    Primary,
    Absence,
}

// ── Internal credential storage ───────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ZepStoredCredentials {
    username: String,
    password: String,
}

fn save_zep_credentials_to_keychain(username: &str, password: &str) -> Result<(), ZepError> {
    let payload = serde_json::to_string(&ZepStoredCredentials {
        username: username.to_string(),
        password: password.to_string(),
    })
    .map_err(|e| ZepError {
        code: ZepErrorCode::KeychainError,
        user_message: "Die ZEP-Zugangsdaten konnten nicht gespeichert werden.".to_string(),
        technical_message: format!("Serialisierung fehlgeschlagen: {e}"),
    })?;

    secret_manager::set_token(ZEP_KEYCHAIN_SERVICE, ZEP_KEYCHAIN_ACCOUNT, &payload).map_err(
        |e| ZepError {
            code: ZepErrorCode::KeychainError,
            user_message: "Die ZEP-Zugangsdaten konnten nicht im Keychain gespeichert werden (Zugriff verweigert?).".to_string(),
            technical_message: e.to_string(),
        },
    )
}

pub(crate) fn load_zep_credentials_from_keychain() -> Result<ZepStoredCredentials, ZepError> {
    let json_str = secret_manager::get_token(ZEP_KEYCHAIN_SERVICE, ZEP_KEYCHAIN_ACCOUNT).map_err(
        |e| match e {
            secret_manager::SecretError::NotFound => ZepError {
                code: ZepErrorCode::MissingCredentials,
                user_message:
                    "Keine ZEP-Zugangsdaten hinterlegt. Bitte ZEP-Verbindung konfigurieren."
                        .to_string(),
                technical_message: "Kein Keychain-Eintrag für ZEP-Zugangsdaten.".to_string(),
            },
            _ => ZepError {
                code: ZepErrorCode::KeychainError,
                user_message:
                    "Auf die ZEP-Zugangsdaten im Keychain konnte nicht zugegriffen werden."
                        .to_string(),
                technical_message: e.to_string(),
            },
        },
    )?;

    serde_json::from_str::<ZepStoredCredentials>(&json_str).map_err(|e| ZepError {
        code: ZepErrorCode::KeychainError,
        user_message: "Die gespeicherten ZEP-Zugangsdaten sind beschädigt. Bitte neu eingeben."
            .to_string(),
        technical_message: format!("Deserialisierung fehlgeschlagen: {e}"),
    })
}

// ── CalDAV HTTP helpers ───────────────────────────────────────────────────────

const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
  </d:prop>
</d:propfind>"#;

/// Issue a PROPFIND request with Basic Auth and return the XML response body.
async fn propfind(url: &str, username: &str, password: &str) -> Result<String, ZepError> {
    let client = reqwest::Client::new();
    let response = client
        .request(
            Method::from_bytes(b"PROPFIND").expect("PROPFIND is a valid HTTP method"),
            url,
        )
        .basic_auth(username, Some(password))
        .header("Depth", "1")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(PROPFIND_BODY)
        .send()
        .await
        .map_err(|e| ZepError {
            code: ZepErrorCode::NetworkError,
            user_message: "ZEP CalDAV-Server ist nicht erreichbar.".to_string(),
            technical_message: format!("PROPFIND fehlgeschlagen für {url}: {e}"),
        })?;

    let status = response.status().as_u16();
    match status {
        401 => {
            return Err(ZepError {
                code: ZepErrorCode::Unauthorized,
                user_message: "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen."
                    .to_string(),
                technical_message: format!("PROPFIND returned HTTP 401 for {url}"),
            });
        }
        404 => {
            return Err(ZepError {
                code: ZepErrorCode::NotFound,
                user_message: "ZEP CalDAV-URL nicht gefunden. Root-URL prüfen.".to_string(),
                technical_message: format!("PROPFIND returned HTTP 404 for {url}"),
            });
        }
        200..=299 => {}
        _ => {
            return Err(ZepError {
                code: ZepErrorCode::NetworkError,
                user_message: "ZEP CalDAV-Server hat einen Fehler zurückgegeben.".to_string(),
                technical_message: format!("PROPFIND returned HTTP {status} for {url}"),
            });
        }
    }

    response.text().await.map_err(|e| ZepError {
        code: ZepErrorCode::InvalidResponse,
        user_message: "Die Antwort des ZEP CalDAV-Servers konnte nicht gelesen werden.".to_string(),
        technical_message: format!("Response body read fehlgeschlagen: {e}"),
    })
}

/// Issue a GET with Basic Auth to test a calendar URL and verify iCal content.
async fn get_calendar(url: &str, username: &str, password: &str) -> Result<(), ZepError> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .basic_auth(username, Some(password))
        .send()
        .await
        .map_err(|e| ZepError {
            code: ZepErrorCode::NetworkError,
            user_message: "Verbindung Zeitüberschreitung. Bitte Verbindung prüfen.".to_string(),
            technical_message: format!("GET fehlgeschlagen für {url}: {e}"),
        })?;

    let status = response.status().as_u16();
    match status {
        401 => {
            return Err(ZepError {
                code: ZepErrorCode::Unauthorized,
                user_message: "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen."
                    .to_string(),
                technical_message: format!("GET returned HTTP 401 for {url}"),
            });
        }
        404 => {
            return Err(ZepError {
                code: ZepErrorCode::NotFound,
                user_message: "Kalender nicht gefunden. Kalender-Zuweisung prüfen.".to_string(),
                technical_message: format!("GET returned HTTP 404 for {url}"),
            });
        }
        200..=299 => {}
        _ => {
            return Err(ZepError {
                code: ZepErrorCode::NetworkError,
                user_message: "ZEP CalDAV-Server hat einen Fehler zurückgegeben.".to_string(),
                technical_message: format!("GET returned HTTP {status} for {url}"),
            });
        }
    }

    let body = response.text().await.map_err(|e| ZepError {
        code: ZepErrorCode::InvalidResponse,
        user_message: "Die Antwort des ZEP-Kalenders konnte nicht gelesen werden.".to_string(),
        technical_message: format!("Response body read fehlgeschlagen: {e}"),
    })?;

    if !body.contains("BEGIN:VCALENDAR") {
        return Err(ZepError {
            code: ZepErrorCode::InvalidResponse,
            user_message: "Ungültige Antwort. Keine gültige iCal-Datei.".to_string(),
            technical_message: format!(
                "Response does not contain BEGIN:VCALENDAR (first 100 chars: {})",
                body.chars().take(100).collect::<String>()
            ),
        });
    }

    Ok(())
}

// ── PROPFIND XML parsing ──────────────────────────────────────────────────────

/// Parse CalDAV PROPFIND multistatus XML and extract calendars.
/// Uses simple string matching to avoid an XML parser dependency.
pub(crate) fn parse_propfind_calendars(body: &str, root_url: &str) -> Vec<ZepCalendar> {
    let base_origin = extract_origin(root_url);
    collect_response_blocks(body)
        .into_iter()
        .filter_map(|block| {
            let is_calendar = block.contains("calendar/>")
                || block.contains("calendar />")
                || block.contains(":calendar/>")
                || block.contains(":calendar />");
            if !is_calendar {
                return None;
            }
            let href = extract_xml_text(block, "href")?;
            let display_name = extract_xml_text(block, "displayname")?;

            let url = if href.starts_with("http://") || href.starts_with("https://") {
                href
            } else {
                format!("{}{}", base_origin, href)
            };

            Some(ZepCalendar { display_name, url })
        })
        .collect()
}

/// Split XML body into individual `<*:response>` or `<response>` blocks.
fn collect_response_blocks(body: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut remaining = body;

    while !remaining.is_empty() {
        let Some(start) = find_response_open(remaining) else {
            break;
        };
        let after_open = &remaining[start..];
        let Some(end) = find_response_close(after_open) else {
            break;
        };
        blocks.push(&after_open[..end]);
        remaining = &after_open[end..];
    }

    blocks
}

/// Find the byte offset of the next `<*response` open tag.
fn find_response_open(text: &str) -> Option<usize> {
    // Match <response or <PREFIX:response
    let mut search = text;
    let mut offset = 0;
    while let Some(lt) = search.find('<') {
        let after_lt = &search[lt + 1..];
        let tag_content = after_lt.splitn(2, '>').next().unwrap_or("");
        let local = tag_content
            .split(':')
            .last()
            .unwrap_or("")
            .split_whitespace()
            .next()
            .unwrap_or("");
        if local == "response" && !tag_content.starts_with('/') {
            return Some(offset + lt);
        }
        offset += lt + 1;
        search = after_lt;
    }
    None
}

/// Find the byte offset past the closing `</response>` or `</PREFIX:response>` tag.
fn find_response_close(text: &str) -> Option<usize> {
    let mut search = text;
    let mut offset = 0;
    while let Some(lt) = search.find("</") {
        let after = &search[lt + 2..];
        let tag_content = after.splitn(2, '>').next().unwrap_or("");
        let local = tag_content.split(':').last().unwrap_or("").trim();
        if local == "response" {
            let end = offset + lt + 2 + tag_content.len() + 1;
            return Some(end);
        }
        offset += lt + 2;
        search = after;
    }
    None
}

/// Extract text content of an XML element by local name (ignoring namespace prefix).
pub(crate) fn extract_xml_text(text: &str, tag: &str) -> Option<String> {
    // Try without namespace prefix: <tag>content</tag>
    let direct_open = format!("<{tag}>");
    if let Some(start) = text.find(&direct_open) {
        let after = &text[start + direct_open.len()..];
        let close = format!("</{tag}>");
        if let Some(end) = after.find(&close) {
            let content = after[..end].trim().to_string();
            return if content.is_empty() {
                None
            } else {
                Some(content)
            };
        }
    }

    // Try with namespace prefix: <prefix:tag>content</prefix:tag>
    let ns_open_suffix = format!(":{tag}>");
    let mut search = text;
    while let Some(colon_pos) = search.find(&ns_open_suffix) {
        let after_open = &search[colon_pos + ns_open_suffix.len()..];
        let close_suffix = format!(":{tag}>");
        if let Some(close_colon) = after_open.find(&close_suffix) {
            if let Some(slash_pos) = after_open[..close_colon].rfind("</") {
                let content = after_open[..slash_pos].trim().to_string();
                return if content.is_empty() {
                    None
                } else {
                    Some(content)
                };
            }
        }
        search = &search[colon_pos + ns_open_suffix.len()..];
    }

    None
}

/// Extract scheme + host from a URL ("https://app.zep.de/caldav/admin" → "https://app.zep.de").
fn extract_origin(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let after_scheme = &url[scheme_end + 3..];
        let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
        return format!("{}://{}", &url[..scheme_end], &after_scheme[..host_end]);
    }
    url.to_string()
}

fn current_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Save ZEP admin credentials: root URL stored in local store; username+password in keychain.
#[tauri::command]
#[specta::specta]
pub fn zep_save_credentials(
    app: tauri::AppHandle,
    root_url: String,
    username: String,
    password: String,
) -> Result<(), ZepError> {
    let root_url = root_url.trim().trim_end_matches('/').to_string();
    if root_url.is_empty() {
        return Err(ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: "Die ZEP CalDAV-URL darf nicht leer sein.".to_string(),
            technical_message: "root_url is empty".to_string(),
        });
    }

    save_zep_credentials_to_keychain(&username, &password)?;

    let mut store =
        crate::integrations::local_store::load_local_store(app.clone()).map_err(|e| ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: e.user_message,
            technical_message: e.technical_message,
        })?;
    store.api_endpoints.zep_caldav_root_url = root_url;
    crate::integrations::local_store::save_local_store(app, store).map_err(|e| ZepError {
        code: ZepErrorCode::InvalidConfiguration,
        user_message: e.user_message,
        technical_message: e.technical_message,
    })?;

    Ok(())
}

/// Load publicly visible ZEP credential info (root URL + username, no password).
/// Returns None if not yet configured.
#[tauri::command]
#[specta::specta]
pub fn zep_load_credentials(app: tauri::AppHandle) -> Result<Option<ZepCredentialsInfo>, ZepError> {
    let store = crate::integrations::local_store::load_local_store(app).map_err(|e| ZepError {
        code: ZepErrorCode::InvalidConfiguration,
        user_message: e.user_message,
        technical_message: e.technical_message,
    })?;

    let root_url = store.api_endpoints.zep_caldav_root_url.trim().to_string();
    if root_url.is_empty() {
        return Ok(None);
    }

    match load_zep_credentials_from_keychain() {
        Ok(creds) => Ok(Some(ZepCredentialsInfo {
            root_url,
            username: creds.username,
        })),
        Err(e) if e.code == ZepErrorCode::MissingCredentials => Ok(None),
        Err(e) => Err(e),
    }
}

/// Test ZEP admin credentials by issuing a PROPFIND. Returns the calendar count on success.
/// Does NOT save credentials — call `zep_save_credentials` after a successful test.
#[tauri::command]
#[specta::specta]
pub async fn zep_test_credentials(
    root_url: String,
    username: String,
    password: String,
) -> Result<ZepCredentialTestResult, ZepError> {
    let root_url = root_url.trim().trim_end_matches('/').to_string();
    if root_url.is_empty() {
        return Err(ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: "Die ZEP CalDAV-URL darf nicht leer sein.".to_string(),
            technical_message: "root_url is empty".to_string(),
        });
    }

    let body = propfind(&root_url, &username, &password).await?;
    let calendars = parse_propfind_calendars(&body, &root_url);

    Ok(ZepCredentialTestResult {
        calendar_count: calendars.len() as u32,
    })
}

/// Discover all CalDAV calendars using the stored admin credentials.
/// The frontend should cache the result for the session and call this on-demand.
#[tauri::command]
#[specta::specta]
pub async fn zep_discover_calendars(app: tauri::AppHandle) -> Result<Vec<ZepCalendar>, ZepError> {
    let store = crate::integrations::local_store::load_local_store(app).map_err(|e| ZepError {
        code: ZepErrorCode::InvalidConfiguration,
        user_message: e.user_message,
        technical_message: e.technical_message,
    })?;

    let root_url = store.api_endpoints.zep_caldav_root_url.trim().to_string();
    if root_url.is_empty() {
        return Err(ZepError {
            code: ZepErrorCode::MissingCredentials,
            user_message: "ZEP CalDAV-URL nicht konfiguriert. Bitte ZEP-Verbindung einrichten."
                .to_string(),
            technical_message: "zep_caldav_root_url is empty in local store".to_string(),
        });
    }

    let creds = load_zep_credentials_from_keychain()?;
    let body = propfind(&root_url, &creds.username, &creds.password).await?;
    let calendars = parse_propfind_calendars(&body, &root_url);

    Ok(calendars)
}

/// Save a ZEP calendar URL for one source (Primary or Absence) and test the connection.
///
/// Steps:
/// 1. Validate calendar_url is not None/empty
/// 2. Sync to Daylite (abort on failure, local store unchanged)
/// 3. Save calendar URL to local store, clear old timestamp
/// 4. Run CalDAV GET with admin credentials
/// 5. Store result timestamp
#[tauri::command]
#[specta::specta]
pub async fn zep_save_and_test_calendar(
    app: tauri::AppHandle,
    daylite_contact_reference: String,
    source: IcalSource,
    calendar_url: Option<String>,
) -> Result<ZepCalendarTestResult, ZepError> {
    // Step 1: Validate selection
    let calendar_url = calendar_url
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);

    let Some(ref cal_url) = calendar_url else {
        return Err(ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: "Bitte einen Kalender auswählen.".to_string(),
            technical_message: "calendar_url is None or empty".to_string(),
        });
    };

    // Step 2: Sync to Daylite — look up the "other" calendar URL to preserve it
    let store =
        crate::integrations::local_store::load_local_store(app.clone()).map_err(|e| ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: e.user_message,
            technical_message: e.technical_message,
        })?;

    let current_setting =
        find_or_default_setting(&store.employee_settings, &daylite_contact_reference);
    let (primary_url, absence_url) = match source {
        IcalSource::Primary => (
            cal_url.clone(),
            current_setting
                .zep_absence_calendar
                .clone()
                .unwrap_or_default(),
        ),
        IcalSource::Absence => (
            current_setting
                .zep_primary_calendar
                .clone()
                .unwrap_or_default(),
            cal_url.clone(),
        ),
    };

    sync_contact_ical_urls(
        app.clone(),
        DayliteUpdateContactIcalUrlsInput {
            contact_reference: daylite_contact_reference.clone(),
            primary_ical_url: primary_url,
            absence_ical_url: absence_url,
        },
    )
    .await
    .map_err(|e| ZepError {
        code: ZepErrorCode::DayliteSyncFailed,
        user_message: format!("Daylite-Synchronisation fehlgeschlagen: {}", e.user_message),
        technical_message: e.technical_message,
    })?;

    // Step 3: Save calendar URL to local store, clear old timestamp
    let mut store =
        crate::integrations::local_store::load_local_store(app.clone()).map_err(|e| ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: e.user_message,
            technical_message: e.technical_message,
        })?;

    update_setting(
        &mut store.employee_settings,
        &daylite_contact_reference,
        |s| match source {
            IcalSource::Primary => {
                s.zep_primary_calendar = Some(cal_url.clone());
                s.primary_ical_last_tested_at = None;
                s.primary_ical_last_test_passed = None;
            }
            IcalSource::Absence => {
                s.zep_absence_calendar = Some(cal_url.clone());
                s.absence_ical_last_tested_at = None;
                s.absence_ical_last_test_passed = None;
            }
        },
    );

    crate::integrations::local_store::save_local_store(app.clone(), store).map_err(|e| {
        ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: e.user_message,
            technical_message: e.technical_message,
        }
    })?;

    // Step 4: Run CalDAV GET test
    let creds = load_zep_credentials_from_keychain()?;
    let timestamp = current_timestamp();
    let test_result = get_calendar(cal_url, &creds.username, &creds.password).await;
    let (success, error_message) = match &test_result {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.user_message.clone())),
    };

    // Step 5: Store result timestamp
    let mut store =
        crate::integrations::local_store::load_local_store(app.clone()).map_err(|e| ZepError {
            code: ZepErrorCode::InvalidConfiguration,
            user_message: e.user_message,
            technical_message: e.technical_message,
        })?;

    update_setting(
        &mut store.employee_settings,
        &daylite_contact_reference,
        |s| match source {
            IcalSource::Primary => {
                s.primary_ical_last_tested_at = Some(timestamp.clone());
                s.primary_ical_last_test_passed = Some(success);
            }
            IcalSource::Absence => {
                s.absence_ical_last_tested_at = Some(timestamp.clone());
                s.absence_ical_last_test_passed = Some(success);
            }
        },
    );

    crate::integrations::local_store::save_local_store(app, store).map_err(|e| ZepError {
        code: ZepErrorCode::InvalidConfiguration,
        user_message: e.user_message,
        technical_message: e.technical_message,
    })?;

    Ok(ZepCalendarTestResult {
        success,
        timestamp,
        error_message,
    })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn find_or_default_setting(
    settings: &[EmployeeSetting],
    daylite_contact_reference: &str,
) -> EmployeeSetting {
    settings
        .iter()
        .find(|s| s.daylite_contact_reference == daylite_contact_reference)
        .cloned()
        .unwrap_or_else(|| EmployeeSetting {
            employee_id: daylite_contact_reference.to_string(),
            daylite_contact_reference: daylite_contact_reference.to_string(),
            ..Default::default()
        })
}

fn update_setting(
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
            employee_id: daylite_contact_reference.to_string(),
            daylite_contact_reference: daylite_contact_reference.to_string(),
            ..Default::default()
        };
        update(&mut new_setting);
        settings.push(new_setting);
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_origin_parses_https_url_with_path() {
        assert_eq!(
            extract_origin("https://app.zep.de/caldav/admin"),
            "https://app.zep.de"
        );
    }

    #[test]
    fn extract_origin_handles_url_without_path() {
        assert_eq!(extract_origin("https://app.zep.de"), "https://app.zep.de");
    }

    #[test]
    fn extract_xml_text_handles_direct_tag() {
        let xml = "<displayname>John Doe - Einsatz</displayname>";
        assert_eq!(
            extract_xml_text(xml, "displayname"),
            Some("John Doe - Einsatz".to_string())
        );
    }

    #[test]
    fn extract_xml_text_handles_namespaced_tag() {
        let xml = "<d:displayname>Max Muster</d:displayname>";
        assert_eq!(
            extract_xml_text(xml, "displayname"),
            Some("Max Muster".to_string())
        );
    }

    #[test]
    fn extract_xml_text_returns_none_for_empty_content() {
        let xml = "<d:displayname>   </d:displayname>";
        assert_eq!(extract_xml_text(xml, "displayname"), None);
    }

    #[test]
    fn parse_propfind_calendars_extracts_calendar_entries() {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/caldav/admin/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>admin</d:displayname>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/caldav/admin/john-einsatz/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>John Doe - Einsatz</d:displayname>
        <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
  <d:response>
    <d:href>/caldav/admin/john-abwesenheit/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>John Doe - Abwesenheit</d:displayname>
        <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

        let calendars = parse_propfind_calendars(body, "https://app.zep.de/caldav/admin");

        assert_eq!(calendars.len(), 2);
        assert_eq!(calendars[0].display_name, "John Doe - Einsatz");
        assert_eq!(
            calendars[0].url,
            "https://app.zep.de/caldav/admin/john-einsatz/"
        );
        assert_eq!(calendars[1].display_name, "John Doe - Abwesenheit");
        assert_eq!(
            calendars[1].url,
            "https://app.zep.de/caldav/admin/john-abwesenheit/"
        );
    }

    #[test]
    fn parse_propfind_skips_non_calendar_collections() {
        let body = r#"<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/caldav/admin/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>Root</d:displayname>
        <d:resourcetype><d:collection/></d:resourcetype>
      </d:prop>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

        let calendars = parse_propfind_calendars(body, "https://app.zep.de/caldav/admin");
        assert!(calendars.is_empty());
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
            employee_id: "/v1/contacts/42".to_string(),
            daylite_contact_reference: "/v1/contacts/42".to_string(),
            zep_primary_calendar: None,
            zep_absence_calendar: None,
            primary_ical_last_tested_at: Some("2026-01-01T00:00:00Z".to_string()),
            absence_ical_last_tested_at: None,
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
