use std::collections::HashMap;

use super::super::types::{CalendarCellEvent, CalendarEventKind, PendingEvent};
use crate::integrations::daylite::projects::ResolvedProject;

pub(crate) fn resolve_event(
    pending: PendingEvent,
    cached_projects: &HashMap<String, ResolvedProject>,
) -> CalendarCellEvent {
    let PendingEvent {
        uid,
        date,
        summary,
        project_ref,
        start_time,
        end_time,
        href,
        order_index,
    } = pending;

    let href = if href.is_empty() { None } else { Some(href) };

    let Some(project_ref) = project_ref else {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Bare,
            title: summary,
            project_status: None,
            project_category: None,
            project_ref: None,
            date,
            start_time,
            end_time,
            href,
            order_index,
            project_pending: false,
        };
    };

    if let Some(resolved) = cached_projects.get(&project_ref) {
        return CalendarCellEvent {
            uid,
            kind: CalendarEventKind::Assignment,
            title: resolved.name.clone(),
            project_status: Some(resolved.status.clone()),
            project_category: resolved.category.clone(),
            project_ref: Some(project_ref.clone()),
            date,
            start_time,
            end_time,
            href,
            order_index,
            project_pending: false,
        };
    }

    CalendarCellEvent {
        uid,
        kind: CalendarEventKind::Assignment,
        title: summary,
        project_status: None,
        project_category: None,
        project_ref: Some(project_ref),
        date,
        start_time,
        end_time,
        href,
        order_index,
        project_pending: true,
    }
}

#[cfg(test)]
mod tests {
    use super::super::classify::classify_event;
    use super::*;
    use crate::integrations::calendar::types::RawVEvent;

    fn resolved(
        project_ref: &str,
        name: &str,
        status: &str,
        category: Option<&str>,
    ) -> HashMap<String, ResolvedProject> {
        HashMap::from([(
            project_ref.to_string(),
            ResolvedProject {
                name: name.to_string(),
                status: status.to_string(),
                category: category.map(str::to_string),
            },
        )])
    }

    fn pending(summary: &str, project_ref: Option<&str>) -> PendingEvent {
        PendingEvent {
            uid: "uid-1".to_string(),
            date: "2026-01-26".to_string(),
            summary: summary.to_string(),
            project_ref: project_ref.map(str::to_string),
            start_time: None,
            end_time: None,
            href: String::new(),
            order_index: None,
        }
    }

    #[test]
    fn resolves_assignment_event() {
        let pending = pending("Projekt Süd", Some("/v1/projects/4001"));

        let event = resolve_event(
            pending,
            &resolved("/v1/projects/4001", "Projekt Süd", "deferred", None),
        );

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Süd");
        assert_eq!(event.project_status, Some("deferred".to_string()));
        assert_eq!(event.project_category, None);
        assert_eq!(event.date, "2026-01-26");
        assert!(!event.project_pending);
    }

    #[test]
    fn resolves_the_category_daylite_returns_for_the_project() {
        let pending = pending("Projekt Nord", Some("/v1/projects/3001"));

        let event = resolve_event(
            pending,
            &resolved(
                "/v1/projects/3001",
                "Projekt Nord",
                "in_progress",
                Some("Termin FIX geplant"),
            ),
        );

        assert_eq!(
            event.project_category,
            Some("Termin FIX geplant".to_string())
        );
    }

    #[test]
    fn an_uncached_project_keeps_the_summary_and_is_pending() {
        let pending = pending("Projekt Ohne Cache", Some("/v1/projects/9999"));

        let event = resolve_event(pending, &HashMap::new());

        assert_eq!(event.kind, CalendarEventKind::Assignment);
        assert_eq!(event.title, "Projekt Ohne Cache");
        assert_eq!(event.project_status, None);
        assert_eq!(event.project_category, None);
        assert!(event.project_pending);
    }

    #[test]
    fn resolves_bare_event() {
        let pending = pending("Auto Werkstatt", None);

        let event = resolve_event(pending, &HashMap::new());

        assert_eq!(event.kind, CalendarEventKind::Bare);
        assert_eq!(event.title, "Auto Werkstatt");
        assert_eq!(event.project_status, None);
        assert_eq!(event.project_category, None);
        assert!(!event.project_pending);
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
        let pending = classify_event(&event);
        let cell_event = resolve_event(
            pending,
            &resolved("/v1/projects/3001", "Projekt Nord", "in_progress", None),
        );

        assert_eq!(
            cell_event.href,
            Some("/calendars/user/cal/uid-href.ics".to_string())
        );
    }

    #[test]
    fn order_index_propagates_through_classify_and_resolve_to_cell_event() {
        let event = RawVEvent {
            uid: "uid-order".to_string(),
            summary: "Projekt Nord".to_string(),
            description: "daylite:/v1/projects/3001".to_string(),
            dtstart: "2026-05-05".to_string(),
            order_index: Some(1),
            ..Default::default()
        };

        let cell_event = resolve_event(classify_event(&event), &HashMap::new());

        assert_eq!(cell_event.order_index, Some(1));
    }
}
