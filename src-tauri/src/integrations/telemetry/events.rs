use super::redact::sanitize;
use serde::Serialize;
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    LoadLocalStore,
    SaveLocalStore,
    LoadWeekEvents,
    CreateAssignment,
    UpdateAssignment,
    MoveAssignment,
    ReorderAssignment,
    DeleteAssignment,
    GetHolidaysForWeek,
    DayliteConnectRefreshToken,
    DayliteListProjects,
    DayliteSearchProjects,
    DayliteQueryOverdueProjects,
    DayliteProjectCategoryColors,
    DayliteListContacts,
    DayliteListCachedContacts,
    DayliteUpdateContactIcalUrls,
    ZepSaveCredentials,
    ZepLoadCredentials,
    ZepTestCredentials,
    ZepDiscoverCalendars,
    ZepSaveAndTestCalendar,
    DayliteRequest,
    CaldavRead,
    CaldavWrite,
    CaldavProjectLookup,
    HolidayApiRequest,
    KeychainRead,
    FrontendRender,
    FrontendPromiseRejection,
    FrontendUncaughtError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Integration {
    Daylite,
    Zep,
    Holidays,
    LocalStore,
    Keychain,
    Frontend,
}

/// Free text is sanitized on the way in, so no raw message can reach a payload.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryEvent {
    name: &'static str,
    properties: Map<String, Value>,
}

impl TelemetryEvent {
    pub fn operation_completed(
        operation: Operation,
        integration: Integration,
        success: bool,
        duration_ms: u64,
    ) -> Self {
        let mut properties = Map::new();
        insert_dimension(&mut properties, "operation", operation);
        insert_dimension(&mut properties, "integration", integration);
        properties.insert("success".to_string(), Value::Bool(success));
        properties.insert("duration_ms".to_string(), Value::from(duration_ms));

        Self {
            name: "operation_completed",
            properties,
        }
    }

    pub fn error_occurred(
        operation: Operation,
        integration: Integration,
        code: ErrorCode,
        message: Option<&str>,
    ) -> Self {
        let mut properties = Map::new();
        insert_dimension(&mut properties, "operation", operation);
        insert_dimension(&mut properties, "integration", integration);
        properties.insert("code".to_string(), Value::String(code.into_value()));
        if let Some(message) = message {
            properties.insert("message".to_string(), Value::String(sanitize(message)));
        }

        Self {
            name: "error_occurred",
            properties,
        }
    }

    pub fn app_started(duration_ms: u64) -> Self {
        let mut properties = Map::new();
        properties.insert("duration_ms".to_string(), Value::from(duration_ms));

        Self {
            name: "app_started",
            properties,
        }
    }

    pub fn with_context(mut self, context: Option<&str>) -> Self {
        if let Some(context) = context {
            self.properties
                .insert("context".to_string(), Value::String(sanitize(context)));
        }

        self
    }

    pub fn with_http_status(mut self, http_status: Option<u16>) -> Self {
        if let Some(status) = http_status {
            self.properties
                .insert("http_status".to_string(), Value::from(status));
        }

        self
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn properties(&self) -> &Map<String, Value> {
        &self.properties
    }
}

/// A closed dimension: built from a serializable error enum, or sanitized when it
/// originates outside the backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorCode(String);

impl ErrorCode {
    pub fn from_enum<T: Serialize>(code: &T) -> Self {
        Self(
            serde_json::to_value(code)
                .ok()
                .and_then(|value| value.as_str().map(str::to_string))
                .unwrap_or_else(|| "unknown".to_string()),
        )
    }

    pub fn from_untrusted(code: &str) -> Self {
        Self(sanitize(code))
    }

    fn into_value(self) -> String {
        self.0
    }
}

fn insert_dimension<T: Serialize>(properties: &mut Map<String, Value>, key: &str, value: T) {
    if let Ok(serialized) = serde_json::to_value(value) {
        properties.insert(key.to_string(), serialized);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::zep::types::{ZepError, ZepErrorCode};

    fn dimension<T: serde::Serialize>(value: &T) -> String {
        serde_json::to_value(value)
            .expect("dimension should serialize")
            .as_str()
            .expect("dimension should serialize to a string")
            .to_string()
    }

    #[test]
    fn operations_serialize_to_snake_case() {
        assert_eq!(dimension(&Operation::LoadWeekEvents), "load_week_events");
        assert_eq!(dimension(&Operation::CreateAssignment), "create_assignment");
        assert_eq!(
            dimension(&Operation::DayliteSearchProjects),
            "daylite_search_projects"
        );
        assert_eq!(dimension(&Operation::CaldavWrite), "caldav_write");
    }

    #[test]
    fn event_built_from_zep_error_transmits_no_calendar_url() {
        let error = ZepError::new(
            ZepErrorCode::NotFound,
            "Der Kalender wurde nicht gefunden.",
            "PROPFIND https://app.zep.de/caldav/admin/emp-1-primary/ returned 404",
        );

        let event = TelemetryEvent::error_occurred(
            Operation::CaldavRead,
            Integration::Zep,
            ErrorCode::from_enum(&error.code),
            Some(&error.technical_message),
        );

        let payload = serde_json::to_string(event.properties()).expect("payload should serialize");
        assert!(!payload.contains("zep.de"));
        assert!(!payload.contains("emp-1-primary"));
        assert!(payload.contains("PROPFIND"));
        assert!(payload.contains("404"));
        assert_eq!(
            event.properties().get("code"),
            Some(&Value::String("NOT_FOUND".to_string()))
        );
    }

    #[test]
    fn structured_dimensions_carry_no_free_text() {
        let event = TelemetryEvent::operation_completed(
            Operation::LoadWeekEvents,
            Integration::Zep,
            false,
            1234,
        )
        .with_http_status(Some(500));

        let properties = event.properties();
        assert_eq!(properties.len(), 5);
        assert_eq!(
            properties.get("operation"),
            Some(&Value::String("load_week_events".to_string()))
        );
        assert_eq!(properties.get("success"), Some(&Value::Bool(false)));
        assert_eq!(properties.get("duration_ms"), Some(&Value::from(1234)));
        assert_eq!(properties.get("http_status"), Some(&Value::from(500)));
    }

    #[test]
    fn integrations_serialize_to_snake_case() {
        assert_eq!(dimension(&Integration::Daylite), "daylite");
        assert_eq!(dimension(&Integration::Zep), "zep");
        assert_eq!(dimension(&Integration::Holidays), "holidays");
        assert_eq!(dimension(&Integration::LocalStore), "local_store");
        assert_eq!(dimension(&Integration::Keychain), "keychain");
        assert_eq!(dimension(&Integration::Frontend), "frontend");
    }
}
