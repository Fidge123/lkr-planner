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

    fn event(uid: &str, kind: CalendarEventKind, date: &str) -> CalendarCellEvent {
        CalendarCellEvent {
            uid: uid.to_string(),
            kind,
            title: "Termin".to_string(),
            project_status: None,
            category_color: None,
            project_category: None,
            project_ref: None,
            date: date.to_string(),
            start_time: None,
            end_time: None,
            href: None,
            order_index: None,
        }
    }

    #[test]
    fn absence_sorted_before_other_kinds_on_same_day() {
        for other in [CalendarEventKind::Assignment, CalendarEventKind::Bare] {
            let mut events = vec![
                event("other-1", other.clone(), "2026-04-28"),
                event("absence-1", CalendarEventKind::Absence, "2026-04-28"),
            ];

            sort_events_absences_first(&mut events);

            assert_eq!(
                events[0].kind,
                CalendarEventKind::Absence,
                "other={other:?}"
            );
            assert_eq!(events[1].kind, other);
        }
    }

    #[test]
    fn absence_on_different_day_does_not_reorder_other_days() {
        let mut events = vec![
            event(
                "assignment-mon",
                CalendarEventKind::Assignment,
                "2026-04-27",
            ),
            event("absence-tue", CalendarEventKind::Absence, "2026-04-28"),
        ];

        sort_events_absences_first(&mut events);

        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }
}
