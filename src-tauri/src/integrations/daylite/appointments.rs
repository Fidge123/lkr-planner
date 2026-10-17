use chrono::{NaiveDate, NaiveDateTime};
use serde::Deserialize;
use serde_json::{json, Map, Value};

use super::auth_flow::{send_authenticated_json, send_authenticated_request};
use super::client::{DayliteApiClient, DayliteHttpMethod, DayliteHttpRequest};
use super::shared::{
    normalize_http_error, trimmed, trimmed_or_none, with_daylite_tokens, DayliteApiError,
    DayliteSearchResult, DayliteTokenState,
};

/// Marks the appointments this app maintains; appointments without it are never touched.
pub(crate) const APP_MANAGED_KEYWORD: &str = "lkr-planner";
const APPOINTMENT_TYPE: &str = "appointment";
const APPOINTMENT_STATUS: &str = "confirmed";
const APPOINTMENT_SHOW_AS: &str = "free";
const FLOATING_TIMEZONE: &str = "floating";
/// Full records otherwise default to a page of ten.
const SEARCH_LIMIT: u16 = 250;
const LOCAL_TIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

/// The attributes the app owns on an appointment it maintains.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AppointmentFields {
    pub(crate) project_ref: String,
    pub(crate) subject: String,
    pub(crate) local_start: NaiveDateTime,
    pub(crate) local_end: NaiveDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DayliteAppointment {
    pub(crate) reference: String,
    pub(crate) subject: String,
    pub(crate) timezone: Option<String>,
    pub(crate) local_start: Option<NaiveDateTime>,
    pub(crate) local_end: Option<NaiveDateTime>,
    pub(crate) appointment_type: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) show_as: Option<String>,
    pub(crate) invitees: Vec<String>,
    pub(crate) project_refs: Vec<String>,
}

impl DayliteAppointment {
    /// An unparseable ID sorts last, so it is never the copy that is kept.
    pub(crate) fn numeric_id(&self) -> u64 {
        self.reference
            .rsplit('/')
            .next()
            .and_then(|id| id.parse().ok())
            .unwrap_or(u64::MAX)
    }

    /// True when patching these fields would change nothing the app owns.
    pub(crate) fn matches(&self, fields: &AppointmentFields) -> bool {
        self.subject == fields.subject
            && self.timezone.as_deref() == Some(FLOATING_TIMEZONE)
            && self.local_start == Some(fields.local_start)
            && self.local_end == Some(fields.local_end)
            && self.appointment_type.as_deref() == Some(APPOINTMENT_TYPE)
            && self.status.as_deref() == Some(APPOINTMENT_STATUS)
            && self.show_as.as_deref() == Some(APPOINTMENT_SHOW_AS)
            && self.invitees.is_empty()
            && self.project_refs == [fields.project_ref.as_str()]
    }
}

pub(crate) struct DayliteAppointments {
    client: DayliteApiClient,
}

impl DayliteAppointments {
    pub(crate) fn new(base_url: &str, app: &tauri::AppHandle) -> Result<Self, DayliteApiError> {
        Ok(Self {
            client: DayliteApiClient::new(base_url)?.with_telemetry(app),
        })
    }

    pub(crate) async fn search_day(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<DayliteAppointment>, DayliteApiError> {
        with_daylite_tokens(&self.client, |tokens| {
            search_app_managed_appointments_core(&self.client, tokens, date)
        })
        .await
    }

    pub(crate) async fn create(
        &self,
        fields: &AppointmentFields,
    ) -> Result<String, DayliteApiError> {
        with_daylite_tokens(&self.client, |tokens| {
            create_appointment_core(&self.client, tokens, fields)
        })
        .await
    }

    pub(crate) async fn patch(
        &self,
        reference: &str,
        fields: &AppointmentFields,
    ) -> Result<(), DayliteApiError> {
        with_daylite_tokens(&self.client, |tokens| {
            patch_appointment_core(&self.client, tokens, reference, fields)
        })
        .await
    }

    pub(crate) async fn delete(&self, reference: &str) -> Result<(), DayliteApiError> {
        with_daylite_tokens(&self.client, |tokens| {
            delete_appointment_core(&self.client, tokens, reference)
        })
        .await
    }
}

#[derive(Debug, Deserialize)]
struct DayliteAppointmentDto {
    #[serde(rename = "self")]
    reference: String,
    #[serde(default)]
    subject: String,
    #[serde(default)]
    timezone: Option<String>,
    #[serde(default)]
    local_start: Option<String>,
    #[serde(default)]
    local_end: Option<String>,
    #[serde(default, rename = "type")]
    appointment_type: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    show_as: Option<String>,
    #[serde(default)]
    invitations: Option<Vec<DayliteInvitationDto>>,
    #[serde(default)]
    projects: Option<Vec<DayliteProjectLinkDto>>,
}

#[derive(Debug, Deserialize)]
struct DayliteInvitationDto {
    #[serde(default)]
    invitee: String,
}

#[derive(Debug, Deserialize)]
struct DayliteProjectLinkDto {
    #[serde(default)]
    project: String,
}

#[derive(Debug, Deserialize)]
struct DayliteCreatedDto {
    location: String,
}

pub(super) async fn search_app_managed_appointments_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    date: NaiveDate,
) -> Result<Vec<DayliteAppointment>, DayliteApiError> {
    let search_result = send_authenticated_json::<DayliteSearchResult<DayliteAppointmentDto>>(
        client,
        &token_state.access_token,
        DayliteHttpRequest {
            query: vec![
                ("full-records".to_string(), "true".to_string()),
                ("limit".to_string(), SEARCH_LIMIT.to_string()),
            ],
            // `in_range` includes both ends, so ending on the last second keeps the next midnight out.
            body: Some(json!({
                "keywords": { "any": [APP_MANAGED_KEYWORD] },
                "utc_start": { "in_range": [format!("{date}T00:00:00Z"), format!("{date}T23:59:59Z")] },
            })),
            ..DayliteHttpRequest::new(DayliteHttpMethod::Post, "/appointments/_search")
        },
    )
    .await?;

    Ok(search_result
        .results
        .into_iter()
        .map(normalize_appointment)
        .collect())
}

pub(super) async fn create_appointment_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    fields: &AppointmentFields,
) -> Result<String, DayliteApiError> {
    let mut body = owned_fields_body(fields);
    body.insert("keywords".to_string(), json!([APP_MANAGED_KEYWORD]));

    let created = send_authenticated_json::<DayliteCreatedDto>(
        client,
        &token_state.access_token,
        DayliteHttpRequest {
            body: Some(Value::Object(body)),
            ..DayliteHttpRequest::new(DayliteHttpMethod::Post, "/appointments")
        },
    )
    .await?;

    Ok(trimmed(created.location))
}

/// Keywords are left out, so keywords added in Daylite survive.
pub(super) async fn patch_appointment_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    reference: &str,
    fields: &AppointmentFields,
) -> Result<(), DayliteApiError> {
    send_authenticated_request(
        client,
        &token_state.access_token,
        DayliteHttpRequest {
            body: Some(Value::Object(owned_fields_body(fields))),
            ..DayliteHttpRequest::new(DayliteHttpMethod::Patch, api_path(reference))
        },
    )
    .await
}

pub(super) async fn delete_appointment_core(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    reference: &str,
) -> Result<(), DayliteApiError> {
    let path = api_path(reference);
    let response = client
        .send_request(DayliteHttpRequest {
            access_token: Some(token_state.access_token),
            ..DayliteHttpRequest::new(DayliteHttpMethod::Delete, path)
        })
        .await?;

    if response.status == 404 || (200..300).contains(&response.status) {
        return Ok(());
    }

    Err(normalize_http_error(response.status, &response.body, path))
}

/// Invitations go out as an empty list, because Daylite otherwise invites the requesting user or keeps invitees added later.
fn owned_fields_body(fields: &AppointmentFields) -> Map<String, Value> {
    let Value::Object(body) = json!({
        "subject": fields.subject,
        "timezone": FLOATING_TIMEZONE,
        "local_start": fields.local_start.format(LOCAL_TIME_FORMAT).to_string(),
        "local_end": fields.local_end.format(LOCAL_TIME_FORMAT).to_string(),
        "type": APPOINTMENT_TYPE,
        "status": APPOINTMENT_STATUS,
        "show_as": APPOINTMENT_SHOW_AS,
        "invitations": [],
        "projects": [{ "project": fields.project_ref }],
    }) else {
        unreachable!("a JSON object literal is an object");
    };

    body
}

/// The client's base URL already carries the version prefix.
fn api_path(reference: &str) -> &str {
    reference.strip_prefix("/v1").unwrap_or(reference)
}

fn normalize_appointment(appointment: DayliteAppointmentDto) -> DayliteAppointment {
    DayliteAppointment {
        reference: trimmed(appointment.reference),
        subject: trimmed(appointment.subject),
        timezone: trimmed_or_none(appointment.timezone),
        local_start: parse_local_time(appointment.local_start),
        local_end: parse_local_time(appointment.local_end),
        appointment_type: trimmed_or_none(appointment.appointment_type),
        status: trimmed_or_none(appointment.status),
        show_as: trimmed_or_none(appointment.show_as),
        invitees: appointment
            .invitations
            .unwrap_or_default()
            .into_iter()
            .map(|invitation| trimmed(invitation.invitee))
            .collect(),
        project_refs: appointment
            .projects
            .unwrap_or_default()
            .into_iter()
            .map(|link| trimmed(link.project))
            .filter(|reference| !reference.is_empty())
            .collect(),
    }
}

fn parse_local_time(value: Option<String>) -> Option<NaiveDateTime> {
    let value = trimmed_or_none(value)?;
    NaiveDateTime::parse_from_str(value.trim_end_matches('Z'), "%Y-%m-%dT%H:%M:%S%.f").ok()
}

/// A day far from planned work, so recording finds no other app-managed appointment on it.
#[cfg(test)]
pub(super) const CASSETTE_DAY: &str = "2030-01-07";

#[cfg(test)]
pub(super) fn cassette_fields(project_ref: &str, subject: &str, end: &str) -> AppointmentFields {
    let at = |time: &str| {
        NaiveDateTime::parse_from_str(&format!("{CASSETTE_DAY}T{time}"), "%Y-%m-%dT%H:%M")
            .expect("cassette time should parse")
    };
    AppointmentFields {
        project_ref: project_ref.to_string(),
        subject: subject.to_string(),
        local_start: at("08:00"),
        local_end: at(end),
    }
}

#[cfg(test)]
pub(super) fn cassette_day() -> NaiveDate {
    NaiveDate::parse_from_str(CASSETTE_DAY, "%Y-%m-%d").expect("cassette day should parse")
}

/// Leaves Daylite as it found it, apart from the trashed appointment.
#[cfg(test)]
pub(super) async fn record_appointment_lifecycle(
    client: &DayliteApiClient,
    token_state: DayliteTokenState,
    project_ref: &str,
) -> Result<(), DayliteApiError> {
    let reference = create_appointment_core(
        client,
        token_state.clone(),
        &cassette_fields(project_ref, "Testprojekt (Max Müller)", "16:00"),
    )
    .await?;
    search_app_managed_appointments_core(client, token_state.clone(), cassette_day()).await?;
    patch_appointment_core(
        client,
        token_state.clone(),
        &reference,
        &cassette_fields(
            project_ref,
            "Testprojekt (Anna Schmidt, Max Müller)",
            "12:00",
        ),
    )
    .await?;
    delete_appointment_core(client, token_state, &reference).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::daylite::client::DayliteHttpMethod;
    use crate::integrations::daylite::shared::DayliteApiErrorCode;
    use crate::integrations::daylite::test_support::{
        mock_client, mock_response, valid_token_state,
    };
    use serde_json::json;

    fn at(date: &str, time: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&format!("{date}T{time}"), "%Y-%m-%dT%H:%M")
            .expect("test time should parse")
    }

    fn fields() -> AppointmentFields {
        AppointmentFields {
            project_ref: "/v1/projects/42".to_string(),
            subject: "Projekt Nord (Anna Schmidt, Max Müller)".to_string(),
            local_start: at("2026-11-02", "08:00"),
            local_end: at("2026-11-02", "16:00"),
        }
    }

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()
    }

    #[tokio::test]
    async fn create_sends_the_owned_fields_with_the_app_keyword() {
        let (client, transport) = mock_client(vec![Ok(mock_response(
            201,
            r#"{"location":"/v1/appointments/5000"}"#,
        ))]);

        create_appointment_core(&client, valid_token_state(), &fields())
            .await
            .expect("create should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, DayliteHttpMethod::Post);
        assert_eq!(requests[0].path, "/appointments");
        assert_eq!(requests[0].access_token.as_deref(), Some("at"));
        assert_eq!(
            requests[0].body,
            Some(json!({
                "subject": "Projekt Nord (Anna Schmidt, Max Müller)",
                "timezone": "floating",
                "local_start": "2026-11-02T08:00:00",
                "local_end": "2026-11-02T16:00:00",
                "type": "appointment",
                "status": "confirmed",
                "show_as": "free",
                "invitations": [],
                "projects": [{ "project": "/v1/projects/42" }],
                "keywords": ["lkr-planner"],
            }))
        );
    }

    #[tokio::test]
    async fn create_reads_the_new_reference_from_location() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            201,
            r#"{"location":" /v1/appointments/5000 "}"#,
        ))]);

        let reference = create_appointment_core(&client, valid_token_state(), &fields())
            .await
            .expect("create should succeed");

        assert_eq!(reference, "/v1/appointments/5000");
    }

    #[tokio::test]
    async fn a_rejected_create_surfaces_the_status() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            400,
            r#"{"base":[{"error":"invalid_start_end_date"}]}"#,
        ))]);

        let error = create_appointment_core(&client, valid_token_state(), &fields())
            .await
            .expect_err("a rejected create should fail");

        assert_eq!(error.http_status, Some(400));
    }

    #[tokio::test]
    async fn search_requests_the_full_app_managed_records_of_the_day() {
        let (client, transport) = mock_client(vec![Ok(mock_response(200, "{}"))]);

        search_app_managed_appointments_core(&client, valid_token_state(), day())
            .await
            .expect("search should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, DayliteHttpMethod::Post);
        assert_eq!(requests[0].path, "/appointments/_search");
        assert_eq!(
            requests[0].query,
            vec![
                ("full-records".to_string(), "true".to_string()),
                ("limit".to_string(), "250".to_string()),
            ]
        );
        assert_eq!(
            requests[0].body,
            Some(json!({
                "keywords": { "any": ["lkr-planner"] },
                "utc_start": {
                    "in_range": ["2026-11-02T00:00:00Z", "2026-11-02T23:59:59Z"]
                },
            }))
        );
    }

    #[tokio::test]
    async fn search_parses_the_attributes_the_app_owns() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[{
                "self":" /v1/appointments/5000 ",
                "subject":" Projekt Nord (Max Müller) ",
                "timezone":"floating",
                "local_start":"2026-11-02T08:00:00",
                "local_end":"2026-11-02T12:00:00",
                "utc_start":"2026-11-02T08:00:00Z",
                "type":"appointment",
                "status":"tentative",
                "show_as":"busy",
                "invitations":[{"invitee":"/v1/users/1000","status":"pending","type":"internal"}],
                "projects":[{"project":"/v1/projects/42"}],
                "keywords":["lkr-planner"]
            }]}"#,
        ))]);

        let appointments =
            search_app_managed_appointments_core(&client, valid_token_state(), day())
                .await
                .expect("search should succeed");

        assert_eq!(
            appointments,
            vec![DayliteAppointment {
                reference: "/v1/appointments/5000".to_string(),
                subject: "Projekt Nord (Max Müller)".to_string(),
                timezone: Some("floating".to_string()),
                local_start: Some(at("2026-11-02", "08:00")),
                local_end: Some(at("2026-11-02", "12:00")),
                appointment_type: Some("appointment".to_string()),
                status: Some("tentative".to_string()),
                show_as: Some("busy".to_string()),
                invitees: vec!["/v1/users/1000".to_string()],
                project_refs: vec!["/v1/projects/42".to_string()],
            }]
        );
    }

    #[tokio::test]
    async fn search_reads_null_lists_as_empty() {
        let (client, _) = mock_client(vec![Ok(mock_response(
            200,
            r#"{"results":[{
                "self":"/v1/appointments/5000",
                "subject":"Projekt Nord (Max Müller)",
                "local_start":"2026-11-02T08:00:00.000",
                "invitations":null,
                "projects":null
            }]}"#,
        ))]);

        let appointments =
            search_app_managed_appointments_core(&client, valid_token_state(), day())
                .await
                .expect("search should succeed");

        assert!(appointments[0].invitees.is_empty());
        assert!(appointments[0].project_refs.is_empty());
        assert_eq!(appointments[0].local_start, Some(at("2026-11-02", "08:00")));
    }

    #[tokio::test]
    async fn search_without_matches_is_empty() {
        let (client, _) = mock_client(vec![Ok(mock_response(200, "{}"))]);

        let appointments =
            search_app_managed_appointments_core(&client, valid_token_state(), day())
                .await
                .expect("search should succeed");

        assert!(appointments.is_empty());
    }

    #[tokio::test]
    async fn patch_rewrites_the_owned_fields_without_touching_keywords() {
        let (client, transport) = mock_client(vec![Ok(mock_response(204, ""))]);

        patch_appointment_core(
            &client,
            valid_token_state(),
            "/v1/appointments/5000",
            &fields(),
        )
        .await
        .expect("patch should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, DayliteHttpMethod::Patch);
        assert_eq!(requests[0].path, "/appointments/5000");
        assert_eq!(
            requests[0].body,
            Some(json!({
                "subject": "Projekt Nord (Anna Schmidt, Max Müller)",
                "timezone": "floating",
                "local_start": "2026-11-02T08:00:00",
                "local_end": "2026-11-02T16:00:00",
                "type": "appointment",
                "status": "confirmed",
                "show_as": "free",
                "invitations": [],
                "projects": [{ "project": "/v1/projects/42" }],
            }))
        );
    }

    #[tokio::test]
    async fn delete_moves_the_appointment_to_the_trash() {
        let (client, transport) = mock_client(vec![Ok(mock_response(204, ""))]);

        delete_appointment_core(&client, valid_token_state(), "/v1/appointments/5000")
            .await
            .expect("delete should succeed");

        let requests = transport.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method, DayliteHttpMethod::Delete);
        assert_eq!(requests[0].path, "/appointments/5000");
        assert_eq!(requests[0].body, None);
    }

    #[tokio::test]
    async fn deleting_a_missing_appointment_counts_as_removed() {
        let (client, _) = mock_client(vec![Ok(mock_response(404, r#"{"error":"invalid_id"}"#))]);

        delete_appointment_core(&client, valid_token_state(), "/v1/appointments/5000")
            .await
            .expect("a missing appointment is already removed");
    }

    #[tokio::test]
    async fn a_failed_delete_surfaces_the_error() {
        let (client, _) = mock_client(vec![Ok(mock_response(500, ""))]);

        let error = delete_appointment_core(&client, valid_token_state(), "/v1/appointments/5000")
            .await
            .expect_err("a server error should fail the delete");

        assert_eq!(error.code, DayliteApiErrorCode::ServerError);
    }

    #[tokio::test]
    async fn appointment_lifecycle_replays_vcr_cassette() {
        // Recorded by `record_daylite_cassettes_from_live_api` with `DAYLITE_VCR_SCOPE=all`, which needs real Daylite credentials.
        let cassette_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/cassettes/daylite-appointments.json");
        if !cassette_path.exists() {
            eprintln!(
                "skipping appointment_lifecycle_replays_vcr_cassette: cassette not recorded yet"
            );
            return;
        }
        let client = DayliteApiClient::with_replay_cassette("daylite-appointments.json")
            .expect("replay client should be created");
        let tokens = crate::integrations::daylite::test_support::token_state(
            "replay-access-token",
            "replay-refresh-token",
        );

        let found = search_app_managed_appointments_core(&client, tokens.clone(), cassette_day())
            .await
            .expect("search should replay from cassette");
        let [appointment] = found.as_slice() else {
            panic!("the recorded day should hold exactly the created appointment: {found:?}");
        };
        let project_ref = appointment.project_refs[0].clone();
        let created = cassette_fields(&project_ref, "Testprojekt (Max Müller)", "16:00");
        assert!(
            appointment.matches(&created),
            "Daylite should store the owned fields as sent: {appointment:?}"
        );

        let reference = create_appointment_core(&client, tokens.clone(), &created)
            .await
            .expect("create should replay from cassette");
        assert_eq!(reference, appointment.reference);
        patch_appointment_core(
            &client,
            tokens.clone(),
            &reference,
            &cassette_fields(
                &project_ref,
                "Testprojekt (Anna Schmidt, Max Müller)",
                "12:00",
            ),
        )
        .await
        .expect("patch should replay from cassette");
        delete_appointment_core(&client, tokens, &reference)
            .await
            .expect("delete should replay from cassette");
    }
}
