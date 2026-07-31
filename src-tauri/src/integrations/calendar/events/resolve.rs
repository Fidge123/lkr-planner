use std::collections::HashMap;

use super::super::types::{CalendarCellEvent, CalendarEventKind, PendingEvent};
use crate::integrations::daylite::projects::ResolvedProject;
use crate::integrations::local_store::DayliteCache;

pub(crate) fn resolve_event(
    pending: PendingEvent,
    cache: &DayliteCache,
    api_results: &HashMap<String, Option<ResolvedProject>>,
    category_colors: &HashMap<String, String>,
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
            category_color: None,
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
            category_color: category_color(cached.category.as_deref(), category_colors),
            project_ref: Some(project_ref.clone()),
            date,
            start_time,
            end_time,
            href,
        };
    }

    if let Some(Some(resolved)) = api_results.get(&project_ref) {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Assignment,
            title: resolved.name.clone(),
            project_status: Some(resolved.status.clone()),
            category_color: category_color(resolved.category.as_deref(), category_colors),
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
        category_color: None,
        project_ref: Some(project_ref),
        date,
        start_time,
        end_time,
        href,
    }
}

fn category_color(
    category: Option<&str>,
    category_colors: &HashMap<String, String>,
) -> Option<String> {
    category_colors.get(category?).cloned()
}

#[cfg(test)]
mod tests {
    use super::super::absences::map_absence_raw_events_for_week;
    use super::super::classify::classify_event;
    use super::*;
    use crate::integrations::calendar::types::RawVEvent;
    use crate::integrations::local_store::DayliteProjectCacheEntry;
    use chrono::NaiveDate;

    fn cache_with_project(category: Option<&str>) -> DayliteCache {
        DayliteCache {
            last_synced_at: None,
            projects: vec![DayliteProjectCacheEntry {
                reference: "/v1/projects/3001".to_string(),
                name: "Projekt Nord".to_string(),
                status: "in_progress".to_string(),
                category: category.map(str::to_string),
            }],
            contacts: vec![],
        }
    }

    fn category_colors(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(name, color)| (name.to_string(), color.to_string()))
            .collect()
    }

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
        let cache = cache_with_project(None);
        let api_results = HashMap::new();

        let event = resolve_event(pending, &cache, &api_results, &HashMap::new());

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Nord");
        assert_eq!(event.project_status, Some("in_progress".to_string()));
        assert_eq!(event.date, "2026-01-26");
    }

    #[test]
    fn resolves_category_color_from_cache() {
        let pending = PendingEvent {
            uid: "uid-1".to_string(),
            date: "2026-01-26".to_string(),
            summary: "Projekt Nord".to_string(),
            project_ref: Some("/v1/projects/3001".to_string()),
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = cache_with_project(Some("Bau"));

        let event = resolve_event(
            pending,
            &cache,
            &HashMap::new(),
            &category_colors(&[("Bau", "#8bc34a")]),
        );

        assert_eq!(event.category_color, Some("#8bc34a".to_string()));
    }

    #[test]
    fn leaves_category_color_unset_when_the_category_has_no_color() {
        let pending = PendingEvent {
            uid: "uid-1".to_string(),
            date: "2026-01-26".to_string(),
            summary: "Projekt Nord".to_string(),
            project_ref: Some("/v1/projects/3001".to_string()),
            start_time: None,
            end_time: None,
            href: String::new(),
        };
        let cache = cache_with_project(Some("Ohne Farbe"));

        let event = resolve_event(
            pending,
            &cache,
            &HashMap::new(),
            &category_colors(&[("Bau", "#8bc34a")]),
        );

        assert_eq!(event.category_color, None);
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
            Some(ResolvedProject {
                name: "Projekt Süd".to_string(),
                status: "deferred".to_string(),
                category: None,
            }),
        );

        let event = resolve_event(pending, &cache, &api_results, &HashMap::new());

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Süd");
        assert_eq!(event.project_status, Some("deferred".to_string()));
        assert_eq!(event.category_color, None);
    }

    #[test]
    fn resolves_category_color_from_api_result() {
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
            Some(ResolvedProject {
                name: "Projekt Süd".to_string(),
                status: "deferred".to_string(),
                category: Some("Wartung".to_string()),
            }),
        );

        let event = resolve_event(
            pending,
            &cache,
            &api_results,
            &category_colors(&[("Wartung", "#03a9f4")]),
        );

        assert_eq!(event.category_color, Some("#03a9f4".to_string()));
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

        let event = resolve_event(
            pending,
            &cache,
            &api_results,
            &category_colors(&[("Bau", "#8bc34a")]),
        );

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert!(event
            .title
            .contains("Beschreibung für Unbekanntes Projekt konnte nicht abgerufen werden"));
        assert_eq!(event.project_status, None);
        assert_eq!(event.category_color, None);
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

        let event = resolve_event(
            pending,
            &cache,
            &api_results,
            &category_colors(&[("Bau", "#8bc34a")]),
        );

        assert_eq!(event.kind, CalendarEventKind::Bare);
        assert_eq!(event.title, "Auto Werkstatt");
        assert_eq!(event.project_status, None);
        assert_eq!(event.category_color, None);
    }

    #[test]
    fn absence_events_have_no_category_color() {
        let raw = RawVEvent {
            uid: "abs-1".to_string(),
            summary: "UB".to_string(),
            dtstart: "2026-04-28".to_string(),
            ..Default::default()
        };
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(vec![raw], week_start);

        assert_eq!(events[0].category_color, None);
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
        let cache = cache_with_project(None);

        let pending = classify_event(&event);
        let cell_event = resolve_event(pending, &cache, &HashMap::new(), &HashMap::new());

        assert_eq!(
            cell_event.href,
            Some("/calendars/user/cal/uid-href.ics".to_string())
        );
    }
}
