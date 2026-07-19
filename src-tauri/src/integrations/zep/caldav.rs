use super::types::{ZepCalendar, ZepError, ZepErrorCode};
use std::time::Duration;
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::Method;

const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<d:propfind xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:prop>
    <d:displayname/>
    <d:resourcetype/>
  </d:prop>
</d:propfind>"#;

pub(super) async fn propfind(
    url: &str,
    username: &str,
    password: &str,
) -> Result<String, ZepError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            ZepError::new(
                ZepErrorCode::NetworkError,
                "HTTP-Client konnte nicht initialisiert werden.",
                format!("Client::build fehlgeschlagen: {e}"),
            )
        })?;
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
        .map_err(|e| {
            ZepError::new(
                ZepErrorCode::NetworkError,
                "ZEP CalDAV-Server ist nicht erreichbar.",
                format!("PROPFIND fehlgeschlagen für {url}: {e}"),
            )
        })?;

    let status = response.status().as_u16();
    match status {
        401 => {
            return Err(ZepError::new(
                ZepErrorCode::Unauthorized,
                "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen.",
                format!("PROPFIND returned HTTP 401 for {url}"),
            ));
        }
        404 => {
            return Err(ZepError::new(
                ZepErrorCode::NotFound,
                "ZEP CalDAV-URL nicht gefunden. Root-URL prüfen.",
                format!("PROPFIND returned HTTP 404 for {url}"),
            ));
        }
        200..=299 => {}
        _ => {
            return Err(ZepError::new(
                ZepErrorCode::NetworkError,
                "ZEP CalDAV-Server hat einen Fehler zurückgegeben.",
                format!("PROPFIND returned HTTP {status} for {url}"),
            ));
        }
    }

    response.text().await.map_err(|e| {
        ZepError::new(
            ZepErrorCode::InvalidResponse,
            "Die Antwort des ZEP CalDAV-Servers konnte nicht gelesen werden.",
            format!("Response body read fehlgeschlagen: {e}"),
        )
    })
}

pub(super) async fn probe_calendar(
    url: &str,
    username: &str,
    password: &str,
) -> Result<(), ZepError> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| {
            ZepError::new(
                ZepErrorCode::NetworkError,
                "HTTP-Client konnte nicht initialisiert werden.",
                format!("Client::build fehlgeschlagen: {e}"),
            )
        })?;
    let response = client
        .request(
            Method::from_bytes(b"PROPFIND").expect("PROPFIND is a valid HTTP method"),
            url,
        )
        .basic_auth(username, Some(password))
        .header("Depth", "0")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(PROPFIND_BODY)
        .send()
        .await
        .map_err(|e| {
            ZepError::new(
                ZepErrorCode::NetworkError,
                "ZEP CalDAV-Server ist nicht erreichbar.",
                format!("PROPFIND Depth:0 fehlgeschlagen für {url}: {e}"),
            )
        })?;

    let status = response.status().as_u16();
    match status {
        401 => Err(ZepError::new(
            ZepErrorCode::Unauthorized,
            "Authentifizierung fehlgeschlagen. ZEP-Zugangsdaten prüfen.",
            format!("PROPFIND Depth:0 returned HTTP 401 for {url}"),
        )),
        404 => Err(ZepError::new(
            ZepErrorCode::NotFound,
            "Kalender nicht gefunden. Kalender-Zuweisung prüfen.",
            format!("PROPFIND Depth:0 returned HTTP 404 for {url}"),
        )),
        200..=299 => Ok(()),
        _ => Err(ZepError::new(
            ZepErrorCode::NetworkError,
            "ZEP CalDAV-Server hat einen Fehler zurückgegeben.",
            format!("PROPFIND Depth:0 returned HTTP {status} for {url}"),
        )),
    }
}

pub(super) fn parse_propfind_calendars(body: &str, root_url: &str) -> Vec<ZepCalendar> {
    let base_origin = extract_origin(root_url);
    let Ok(doc) = roxmltree::Document::parse(body) else {
        return vec![];
    };

    doc.root()
        .descendants()
        .filter(|n| n.has_tag_name(("DAV:", "response")))
        .filter_map(|response| {
            let is_calendar = response
                .descendants()
                .any(|n| n.has_tag_name(("urn:ietf:params:xml:ns:caldav", "calendar")));
            if !is_calendar {
                return None;
            }
            let href = response
                .descendants()
                .find(|n| n.has_tag_name(("DAV:", "href")))?
                .text()?
                .trim()
                .to_string();
            if href.is_empty() {
                return None;
            }
            let display_name = response
                .descendants()
                .find(|n| n.has_tag_name(("DAV:", "displayname")))?
                .text()?
                .trim()
                .to_string();
            if display_name.is_empty() {
                return None;
            }
            let url = if href.starts_with("http://") || href.starts_with("https://") {
                href
            } else {
                format!("{}{}", base_origin, href)
            };
            Some(ZepCalendar { display_name, url })
        })
        .collect()
}

fn extract_origin(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let after_scheme = &url[scheme_end + 3..];
        let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
        return format!("{}://{}", &url[..scheme_end], &after_scheme[..host_end]);
    }
    url.to_string()
}

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
    fn parse_propfind_accepts_207_multistatus_body() {
        let body = r#"<?xml version="1.0" encoding="utf-8"?>
<d:multistatus xmlns:d="DAV:" xmlns:c="urn:ietf:params:xml:ns:caldav">
  <d:response>
    <d:href>/caldav/admin/anna-einsatz/</d:href>
    <d:propstat>
      <d:prop>
        <d:displayname>Anna B - Einsatz</d:displayname>
        <d:resourcetype><d:collection/><c:calendar/></d:resourcetype>
      </d:prop>
      <d:status>HTTP/1.1 200 OK</d:status>
    </d:propstat>
  </d:response>
</d:multistatus>"#;

        let calendars = parse_propfind_calendars(body, "https://app.zep.de/caldav/admin");

        assert_eq!(calendars.len(), 1);
        assert_eq!(calendars[0].display_name, "Anna B - Einsatz");
        assert_eq!(
            calendars[0].url,
            "https://app.zep.de/caldav/admin/anna-einsatz/"
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
    fn vcr_typical_multi_employee_response_extracts_all_calendars() {
        let body = include_str!("zep_vcr/propfind_typical_multi_employee.xml");
        let calendars = parse_propfind_calendars(body, "https://app.zep.de/caldav/admin");

        assert_eq!(calendars.len(), 6);
        assert_eq!(calendars[0].display_name, "Max Mustermann - Einsatz");
        assert_eq!(
            calendars[0].url,
            "https://app.zep.de/caldav/admin/max-mustermann-einsatz/"
        );
        assert_eq!(calendars[1].display_name, "Max Mustermann - Abwesenheit");
        assert_eq!(calendars[2].display_name, "Anna Bauer - Einsatz");
        assert_eq!(calendars[3].display_name, "Anna Bauer - Abwesenheit");
        assert_eq!(calendars[4].display_name, "Klaus Weber - Einsatz");
        assert_eq!(calendars[5].display_name, "Klaus Weber - Abwesenheit");
    }

    #[test]
    fn vcr_umlaut_display_names_are_preserved_and_entities_decoded() {
        let body = include_str!("zep_vcr/propfind_umlaut_and_entity_names.xml");
        let calendars = parse_propfind_calendars(body, "https://app.zep.de/caldav/admin");

        assert_eq!(calendars.len(), 3);
        assert_eq!(calendars[0].display_name, "Jörg Schröder - Einsatz");
        // XML entity &amp; must be decoded to & by the parser, not returned literally
        assert_eq!(calendars[1].display_name, "Müller & Söhne - Einsatz");
        assert_eq!(calendars[2].display_name, "Günther Weiß - Einsatz");
    }

    #[test]
    fn vcr_alternate_namespace_prefix_is_handled_correctly() {
        // Uses xmlns:dav="DAV:" and xmlns:caldav="urn:ietf:params:xml:ns:caldav"
        // instead of the typical d:/c: prefixes. The parser matches by namespace
        // URI, not prefix string.
        let body = include_str!("zep_vcr/propfind_alternate_ns_prefix.xml");
        let calendars = parse_propfind_calendars(body, "https://app.zep.de/caldav/admin");

        assert_eq!(calendars.len(), 2);
        assert_eq!(calendars[0].display_name, "Test Mitarbeiter - Einsatz");
        assert_eq!(calendars[1].display_name, "Test Mitarbeiter - Abwesenheit");
    }

    #[test]
    fn parse_propfind_returns_empty_for_invalid_xml() {
        let calendars =
            parse_propfind_calendars("not xml {{ at all", "https://app.zep.de/caldav/admin");
        assert!(calendars.is_empty());
    }
}
