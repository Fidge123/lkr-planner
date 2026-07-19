use super::super::types::{CalendarCellEvent, CalendarEventKind};

pub(crate) fn sort_events_absences_first(events: &mut [CalendarCellEvent]) {
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

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }
}
