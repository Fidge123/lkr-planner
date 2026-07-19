use std::collections::HashMap;

use super::super::types::{CalendarCellEvent, CalendarEventKind, PendingEvent};
use crate::integrations::local_store::DayliteCache;

pub(crate) fn resolve_event(
    pending: PendingEvent,
    cache: &DayliteCache,
    api_results: &HashMap<String, Option<(String, String)>>,
) -> CalendarCellEvent {
    let PendingEvent {
        uid,
        date,
        summary,
        project_ref,
        start_time,
        end_time,
        href,
    } = pending;

    let href = if href.is_empty() { None } else { Some(href) };

    let Some(project_ref) = project_ref else {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Bare,
            title: summary,
            project_status: None,
            project_ref: None,
            date,
            start_time,
            end_time,
            href,
        };
    };

    if let Some(cached) = cache.projects.iter().find(|p| p.reference == project_ref) {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Assignment,
            title: cached.name.clone(),
            project_status: Some(cached.status.clone()),
            project_ref: Some(project_ref.clone()),
            date,
            start_time,
            end_time,
            href,
        };
    }

    if let Some(Some((name, status))) = api_results.get(&project_ref) {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Assignment,
            title: name.clone(),
            project_status: Some(status.clone()),
            project_ref: Some(project_ref.clone()),
            date,
            start_time,
            end_time,
            href,
        };
    }

    CalendarCellEvent {
        uid,
        kind: CalendarEventKind::Assignment,
        title: format!("Beschreibung für {} konnte nicht abgerufen werden", summary),
        project_status: None,
        project_ref: Some(project_ref),
        date,
        start_time,
        end_time,
        href,
    }
}

#[cfg(test)]
mod tests {
    use super::super::classify::classify_event;
    use super::*;
    use crate::integrations::calendar::types::RawVEvent;
    use crate::integrations::local_store::DayliteProjectCacheEntry;

    #[test]
    fn resolves_assignment_event_from_cache() {
        let pending = PendingEvent {
            uid: "uid-1".to_string(),
            date: "2026-01-26".to_string(),
            summary: "Projekt Nord".to_string(),
            project_ref: Some("/v1/projects/3001".to_string()),
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = DayliteCache {
            last_synced_at: None,
            projects: vec![DayliteProjectCacheEntry {
                reference: "/v1/projects/3001".to_string(),
                name: "Projekt Nord".to_string(),
                status: "in_progress".to_string(),
            }],
            contacts: vec![],
        };
        let api_results = HashMap::new();

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Nord");
        assert_eq!(event.project_status, Some("in_progress".to_string()));
        assert_eq!(event.date, "2026-01-26");
    }

    #[test]
    fn resolves_assignment_event_from_api_result() {
        let pending = PendingEvent {
            uid: "uid-2".to_string(),
            date: "2026-01-27".to_string(),
            summary: "Projekt Süd".to_string(),
            project_ref: Some("/v1/projects/4001".to_string()),
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = DayliteCache::default();
        let mut api_results = HashMap::new();
        api_results.insert(
            "/v1/projects/4001".to_string(),
            Some(("Projekt Süd".to_string(), "deferred".to_string())),
        );

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Süd");
        assert_eq!(event.project_status, Some("deferred".to_string()));
    }

    #[test]
    fn shows_placeholder_when_project_not_resolvable() {
        let pending = PendingEvent {
            uid: "uid-3".to_string(),
            date: "2026-01-28".to_string(),
            summary: "Unbekanntes Projekt".to_string(),
            project_ref: Some("/v1/projects/9999".to_string()),
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = DayliteCache::default();
        let mut api_results = HashMap::new();
        api_results.insert("/v1/projects/9999".to_string(), None);

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert!(event
            .title
            .contains("Beschreibung für Unbekanntes Projekt konnte nicht abgerufen werden"));
        assert_eq!(event.project_status, None);
    }

    #[test]
    fn resolves_bare_event() {
        let pending = PendingEvent {
            uid: "uid-4".to_string(),
            date: "2026-01-29".to_string(),
            summary: "Auto Werkstatt".to_string(),
            project_ref: None,
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = DayliteCache::default();
        let api_results = HashMap::new();

        let event = resolve_event(pending, &cache, &api_results);

        assert_eq!(event.kind, CalendarEventKind::Bare);
        assert_eq!(event.title, "Auto Werkstatt");
        assert_eq!(event.project_status, None);
    }

    #[test]
    fn href_propagates_through_classify_and_resolve_to_cell_event() {
        let event = RawVEvent {
            uid: "uid-href".to_string(),
            summary: "Projekt Nord".to_string(),
            description: "daylite:/v1/projects/3001".to_string(),
            dtstart: "2026-05-05".to_string(),
            href: "/calendars/user/cal/uid-href.ics".to_string(),
            ..Default::default()
        };
        let cache = DayliteCache {
            last_synced_at: None,
            projects: vec![DayliteProjectCacheEntry {
                reference: "/v1/projects/3001".to_string(),
                name: "Projekt Nord".to_string(),
                status: "in_progress".to_string(),
            }],
            contacts: vec![],
        };

        let pending = classify_event(event);
        let cell_event = resolve_event(pending, &cache, &HashMap::new());

        assert_eq!(
            cell_event.href,
            Some("/calendars/user/cal/uid-href.ics".to_string())
        );
    }
}
