use chrono::NaiveDate;
use std::collections::HashMap;

use super::types::{CalendarCellEvent, CalendarEventKind, PendingEvent, RawVEvent};
use crate::integrations::local_store::DayliteCache;

const DAYLITE_DESCRIPTION_PREFIX: &str = "daylite:";

// ── Event classification ──────────────────────────────────────────────────────

/// Classifies a raw VEVENT as a lkr-planner assignment or a bare calendar event.
pub(super) fn classify_event(event: RawVEvent) -> PendingEvent {
    let date = event.dtstart;

    let uid = if event.uid.is_empty() {
        // Synthesise a stable-ish UID from event content. Summary is sanitized to alphanumeric
        // and hyphens only, so the UID is safe to embed in keys or URLs.
        let safe: String = event
            .summary
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-')
            .take(50)
            .collect();
        format!("synthetic-{date}-{safe}")
    } else {
        event.uid
    };

    // Strip ASCII whitespace, BOM (U+FEFF), and zero-width space (U+200B) that some
    // calendar UIs prepend to the description field.
    let first_line = event
        .description
        .lines()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}' || c == '\u{200b}');

    let project_ref = if let Some(stripped) = first_line.strip_prefix(DAYLITE_DESCRIPTION_PREFIX) {
        let raw_ref = stripped.trim();
        if raw_ref.is_empty() {
            None
        } else {
            Some(raw_ref.to_string())
        }
    } else {
        None
    };

    PendingEvent {
        uid,
        date,
        summary: event.summary,
        project_ref,
        start_time: event.start_time,
        end_time: event.end_time,
        href: event.href,
    }
}

// ── Project resolution ────────────────────────────────────────────────────────

/// Resolves a pending event into a `CalendarCellEvent` using the Daylite cache and
/// pre-fetched API results. Falls back to a German placeholder if the project cannot
/// be resolved.
pub(super) fn resolve_event(
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

    // Try the local Daylite cache first.
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

    // Try the pre-fetched API result.
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

    // Placeholder: project could not be resolved.
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

// ── Event ordering ────────────────────────────────────────────────────────────

/// Sorts a mixed list of calendar events so that `Absence` events always appear
/// first within each day. Within the same kind, original relative order is preserved.
pub(super) fn sort_events_absences_first(events: &mut [CalendarCellEvent]) {
    events.sort_by(|a, b| {
        let kind_order = |e: &CalendarCellEvent| {
            if matches!(e.kind, CalendarEventKind::Absence) {
                0u8
            } else {
                1u8
            }
        };
        a.date.cmp(&b.date).then(kind_order(a).cmp(&kind_order(b)))
    });
}

// ── Absence event mapping ─────────────────────────────────────────────────────

/// Maps raw absence calendar events to `CalendarCellEvent`s for the given week.
/// All-day events with a `dtend` are expanded into one event per day in
/// `[dtstart, dtend)` clamped to `[week_start, week_start + 7)`.
pub(super) fn map_absence_raw_events_for_week(
    raw_events: Vec<RawVEvent>,
    week_start: NaiveDate,
) -> Vec<CalendarCellEvent> {
    let week_end = week_start + chrono::Duration::days(7);
    let mut result = Vec::new();

    for raw in raw_events {
        let event_start = match NaiveDate::parse_from_str(&raw.dtstart, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => continue,
        };

        let href = if raw.href.is_empty() {
            None
        } else {
            Some(raw.href.clone())
        };

        if let Some(event_end) = raw.dtend {
            // All-day multi-day event: expand into per-day events within the week.
            let clamped_start = event_start.max(week_start);
            let clamped_end = event_end.min(week_end);
            let mut day = clamped_start;
            while day < clamped_end {
                result.push(CalendarCellEvent {
                    // NaiveDate Display format is "yyyy-MM-dd" (RFC 3339 date).
                    uid: format!("{}-{}", raw.uid, day),
                    kind: CalendarEventKind::Absence,
                    title: raw.summary.clone(),
                    project_status: None,
                    project_ref: None,
                    date: day.format("%Y-%m-%d").to_string(),
                    start_time: None,
                    end_time: None,
                    href: href.clone(),
                });
                day += chrono::Duration::days(1);
            }
        } else {
            result.push(CalendarCellEvent {
                uid: raw.uid,
                kind: CalendarEventKind::Absence,
                title: raw.summary,
                project_status: None,
                project_ref: None,
                date: raw.dtstart,
                start_time: raw.start_time,
                end_time: raw.end_time,
                href,
            });
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integrations::local_store::DayliteProjectCacheEntry;

    // ── Event classification ──

    #[test]
    fn classifies_lkr_planner_event_with_daylite_description() {
        let event = RawVEvent {
            uid: "uid-1".to_string(),
            summary: "Projekt Nord".to_string(),
            description: "daylite:/v1/projects/3001".to_string(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, Some("/v1/projects/3001".to_string()));
        assert_eq!(pending.date, "2026-01-26");
        assert_eq!(pending.summary, "Projekt Nord");
    }

    #[test]
    fn classifies_bare_event_without_daylite_description() {
        let event = RawVEvent {
            uid: "uid-2".to_string(),
            summary: "Auto Werkstatt".to_string(),
            description: "Bitte Auto abholen".to_string(),
            dtstart: "2026-01-27".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, None);
        assert_eq!(pending.summary, "Auto Werkstatt");
    }

    #[test]
    fn classifies_bare_event_with_empty_description() {
        let event = RawVEvent {
            uid: "uid-3".to_string(),
            summary: "Blockertermin".to_string(),
            description: String::new(),
            dtstart: "2026-01-28".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, None);
    }

    #[test]
    fn classifies_event_with_multiline_description_using_first_line_only() {
        let event = RawVEvent {
            uid: "uid-4".to_string(),
            summary: "Projekt Süd".to_string(),
            description: "daylite:/v1/projects/4001\nZusätzliche Notizen hier".to_string(),
            dtstart: "2026-01-29".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, Some("/v1/projects/4001".to_string()));
    }

    #[test]
    fn synthesises_uid_for_event_without_uid() {
        let event = RawVEvent {
            uid: String::new(),
            summary: "Ohne UID".to_string(),
            description: String::new(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert!(!pending.uid.is_empty());
        assert!(pending.uid.starts_with("synthetic-"));
    }

    // M5 (red): BOM-prefixed description should still classify as a Daylite event.
    #[test]
    fn classifies_event_with_bom_prefixed_daylite_description() {
        let event = RawVEvent {
            uid: "uid-bom".to_string(),
            summary: "Projekt BOM".to_string(),
            description: "\u{feff}daylite:/v1/projects/5001".to_string(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert_eq!(pending.project_ref, Some("/v1/projects/5001".to_string()));
    }

    // L3 (red): synthetic UID must not contain newlines, slashes, or other special characters.
    #[test]
    fn synthetic_uid_contains_only_safe_characters() {
        let event = RawVEvent {
            uid: String::new(),
            summary: "Termin\nmit/Sonderzeichen".to_string(),
            description: String::new(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        };

        let pending = classify_event(event);

        assert!(!pending.uid.contains('\n'), "UID must not contain newline");
        assert!(!pending.uid.contains('/'), "UID must not contain slash");
    }

    // ── Project resolution ──

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

    // ── Absence event mapping ──

    #[test]
    fn absence_event_has_absence_kind_title_and_no_project_status() {
        let raw = RawVEvent {
            uid: "abs-1".to_string(),
            summary: "Urlaub".to_string(),
            description: String::new(),
            dtstart: "2026-04-28".to_string(),
            ..Default::default()
        };
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(vec![raw], week_start);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[0].title, "Urlaub");
        assert_eq!(events[0].project_status, None);
    }

    #[test]
    fn maps_multiple_absence_events_from_raw() {
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();
        let raw = vec![
            RawVEvent {
                uid: "abs-1".to_string(),
                summary: "Urlaub".to_string(),
                dtstart: "2026-04-28".to_string(),
                ..Default::default()
            },
            RawVEvent {
                uid: "abs-2".to_string(),
                summary: "Krankenstand".to_string(),
                dtstart: "2026-04-29".to_string(),
                ..Default::default()
            },
        ];

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.kind == CalendarEventKind::Absence));
        assert!(events.iter().all(|e| e.project_status.is_none()));
    }

    #[test]
    fn returns_empty_when_no_absence_raw_events() {
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();
        let events = map_absence_raw_events_for_week(vec![], week_start);
        assert!(events.is_empty());
    }

    #[test]
    fn absence_fetch_failure_produces_no_absence_events() {
        // Simulates the silent-failure path: when fetch_calendar_events returns Err,
        // the caller passes an empty vec to the mapping function.
        let raw: Vec<RawVEvent> = Vec::new();
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert!(events.is_empty());
    }

    // ── Multi-day absence expansion ──

    #[test]
    fn multi_day_absence_expands_into_one_event_per_day_in_week() {
        // Mon–Fri absence (DTEND is exclusive: Sat = last day not covered).
        let raw = vec![RawVEvent {
            uid: "abs-1".to_string(),
            summary: "Urlaub".to_string(),
            dtstart: "2026-04-27".to_string(),
            dtend: Some(NaiveDate::from_ymd_opt(2026, 5, 2).unwrap()),
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 5);
        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[4].date, "2026-05-01");
        assert!(events.iter().all(|e| e.kind == CalendarEventKind::Absence));
        assert!(events.iter().all(|e| e.title == "Urlaub"));
    }

    #[test]
    fn multi_day_absence_starting_before_week_only_covers_days_in_week() {
        // Absence starts last week (Mon Apr 20), ends Wed Apr 29 (exclusive).
        // Only Mon Apr 27 and Tue Apr 28 fall in this week.
        let raw = vec![RawVEvent {
            uid: "abs-2".to_string(),
            summary: "Krankenstand".to_string(),
            dtstart: "2026-04-20".to_string(),
            dtend: Some(NaiveDate::from_ymd_opt(2026, 4, 29).unwrap()),
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 2);
        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }

    // H1: DATE-TIME DTSTART with no DATE DTEND → single-day event (intentional; expansion only
    // applies when the iCal source uses VALUE=DATE for DTEND, as ZEP does for all-day absences).
    #[test]
    fn absence_with_timed_dtstart_and_no_dtend_produces_single_event() {
        let raw = vec![RawVEvent {
            uid: "abs-timed".to_string(),
            summary: "Kurzurlaub".to_string(),
            dtstart: "2026-04-28".to_string(),
            dtend: None,
            start_time: Some("08:00".to_string()),
            end_time: Some("17:00".to_string()),
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].date, "2026-04-28");
        assert_eq!(events[0].kind, CalendarEventKind::Absence);
    }

    // L8: absence events with a daylite: description must NOT be classified as assignments.
    #[test]
    fn absence_events_are_never_classified_as_assignments_regardless_of_description() {
        let raw = vec![RawVEvent {
            uid: "abs-daylite".to_string(),
            summary: "Urlaub".to_string(),
            description: "daylite:/v1/projects/9999".to_string(),
            dtstart: "2026-04-28".to_string(),
            dtend: None,
            ..Default::default()
        }];
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[0].project_status, None);
    }

    // ── Event ordering ──

    // Ordering: absence events must appear before other event kinds on the same day.
    #[test]
    fn absence_sorted_before_assignment_on_same_day() {
        let mut events = vec![
            CalendarCellEvent {
                uid: "assignment-1".to_string(),
                kind: CalendarEventKind::Assignment,
                title: "Projekt".to_string(),
                project_status: Some("in_progress".to_string()),
                project_ref: Some("/v1/projects/1".to_string()),
                date: "2026-04-28".to_string(),
                start_time: Some("09:00".to_string()),
                end_time: Some("17:00".to_string()),
                href: None,
            },
            CalendarCellEvent {
                uid: "absence-1".to_string(),
                kind: CalendarEventKind::Absence,
                title: "Urlaub".to_string(),
                project_status: None,
                project_ref: None,
                date: "2026-04-28".to_string(),
                start_time: None,
                end_time: None,
                href: None,
            },
        ];

        sort_events_absences_first(&mut events);

        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[1].kind, CalendarEventKind::Assignment);
    }

    #[test]
    fn absence_sorted_before_bare_event_on_same_day() {
        let mut events = vec![
            CalendarCellEvent {
                uid: "bare-1".to_string(),
                kind: CalendarEventKind::Bare,
                title: "Blocker".to_string(),
                project_status: None,
                project_ref: None,
                date: "2026-04-28".to_string(),
                start_time: Some("10:00".to_string()),
                end_time: None,
                href: None,
            },
            CalendarCellEvent {
                uid: "absence-1".to_string(),
                kind: CalendarEventKind::Absence,
                title: "Urlaub".to_string(),
                project_status: None,
                project_ref: None,
                date: "2026-04-28".to_string(),
                start_time: None,
                end_time: None,
                href: None,
            },
        ];

        sort_events_absences_first(&mut events);

        assert_eq!(events[0].kind, CalendarEventKind::Absence);
        assert_eq!(events[1].kind, CalendarEventKind::Bare);
    }

    #[test]
    fn absence_on_different_day_does_not_reorder_other_days() {
        let mut events = vec![
            CalendarCellEvent {
                uid: "assignment-mon".to_string(),
                kind: CalendarEventKind::Assignment,
                title: "Projekt".to_string(),
                project_status: Some("in_progress".to_string()),
                project_ref: Some("/v1/projects/1".to_string()),
                date: "2026-04-27".to_string(),
                start_time: Some("09:00".to_string()),
                end_time: None,
                href: None,
            },
            CalendarCellEvent {
                uid: "absence-tue".to_string(),
                kind: CalendarEventKind::Absence,
                title: "Urlaub".to_string(),
                project_status: None,
                project_ref: None,
                date: "2026-04-28".to_string(),
                start_time: None,
                end_time: None,
                href: None,
            },
        ];

        sort_events_absences_first(&mut events);

        // Monday assignment stays before Tuesday absence (different days).
        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }
}
