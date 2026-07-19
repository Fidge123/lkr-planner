use chrono::NaiveDate;

use super::super::types::{CalendarCellEvent, CalendarEventKind, RawVEvent};

pub(crate) fn map_absence_raw_events_for_week(
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
            let clamped_start = event_start.max(week_start);
            let clamped_end = event_end.min(week_end);
            let mut day = clamped_start;
            while day < clamped_end {
                result.push(CalendarCellEvent {
                    // NaiveDate's Display is "yyyy-MM-dd", keeping expanded UIDs stable.
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
        let raw: Vec<RawVEvent> = Vec::new();
        let week_start = NaiveDate::from_ymd_opt(2026, 4, 27).unwrap();

        let events = map_absence_raw_events_for_week(raw, week_start);

        assert!(events.is_empty());
    }

    #[test]
    fn multi_day_absence_expands_into_one_event_per_day_in_week() {
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
}
