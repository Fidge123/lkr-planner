use super::super::types::{CalendarCellEvent, CalendarEventKind};
use std::cmp::Reverse;

/// Per day, absences come first and assignments follow the order their slots are allocated in.
/// Bare events keep their place.
pub(crate) fn sort_week_events(events: &mut [CalendarCellEvent]) {
    let is_absence = |e: &CalendarCellEvent| matches!(e.kind, CalendarEventKind::Absence);
    events.sort_by(|a, b| a.date.cmp(&b.date).then(is_absence(b).cmp(&is_absence(a))));

    for day in events.chunk_by_mut(|a, b| a.date == b.date) {
        let slots: Vec<usize> = (0..day.len())
            .filter(|&i| matches!(day[i].kind, CalendarEventKind::Assignment))
            .collect();
        let mut assignments: Vec<CalendarCellEvent> =
            slots.iter().map(|&i| day[i].clone()).collect();
        assignments.sort_by(|a, b| slot_order(a).cmp(&slot_order(b)));
        for (slot, assignment) in slots.into_iter().zip(assignments) {
            day[slot] = assignment;
        }
    }
}

fn slot_order(event: &CalendarCellEvent) -> (u32, u32, std::cmp::Reverse<u32>, &str) {
    ordering_key(
        event.order_index,
        event.start_time.as_deref(),
        event.end_time.as_deref(),
        &event.uid,
    )
}

/// Ordered by persisted index, then earlier start, then longer duration, then UID for a total order.
/// Missing index or times sort last.
pub(crate) fn ordering_key<'a>(
    order_index: Option<u32>,
    start_time: Option<&str>,
    end_time: Option<&str>,
    uid: &'a str,
) -> (u32, u32, Reverse<u32>, &'a str) {
    let start = minute_of_time(start_time);
    let end = minute_of_time(end_time);
    let duration = match (start, end) {
        (Some(start), Some(end)) => end.saturating_sub(start),
        _ => 0,
    };
    (
        order_index.unwrap_or(u32::MAX),
        start.unwrap_or(u32::MAX),
        Reverse(duration),
        uid,
    )
}

/// Minutes since midnight for an `HH:MM` time, `None` for an all-day or malformed one.
fn minute_of_time(time: Option<&str>) -> Option<u32> {
    let (hours, minutes) = time?.split_once(':')?;
    Some(hours.parse::<u32>().ok()? * 60 + minutes.parse::<u32>().ok()?)
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
            project_category: None,
            project_ref: None,
            date: date.to_string(),
            start_time: None,
            end_time: None,
            href: None,
            order_index: None,
            project_pending: false,
        }
    }

    #[test]
    fn absence_sorted_before_other_kinds_on_same_day() {
        for other in [CalendarEventKind::Assignment, CalendarEventKind::Bare] {
            let mut events = vec![
                event("other-1", other.clone(), "2026-04-28"),
                event("absence-1", CalendarEventKind::Absence, "2026-04-28"),
            ];

            sort_week_events(&mut events);

            assert_eq!(
                events[0].kind,
                CalendarEventKind::Absence,
                "other={other:?}"
            );
            assert_eq!(events[1].kind, other);
        }
    }

    fn assignment(uid: &str, order_index: Option<u32>, start: Option<&str>) -> CalendarCellEvent {
        CalendarCellEvent {
            order_index,
            start_time: start.map(str::to_string),
            ..event(uid, CalendarEventKind::Assignment, "2026-04-28")
        }
    }

    fn uids(events: &[CalendarCellEvent]) -> Vec<&str> {
        events.iter().map(|e| e.uid.as_str()).collect()
    }

    #[test]
    fn assignments_follow_the_order_their_slots_are_allocated_in() {
        let mut events = vec![
            assignment("unindexed", None, Some("07:00")),
            assignment("second", Some(1), Some("07:00")),
            assignment("first-late", Some(0), Some("10:00")),
            assignment("first-early", Some(0), Some("08:00")),
        ];

        sort_week_events(&mut events);

        assert_eq!(
            uids(&events),
            ["first-early", "first-late", "second", "unindexed"]
        );
    }

    #[test]
    fn bare_events_keep_their_place_among_the_assignments() {
        let mut events = vec![
            assignment("second", Some(1), None),
            event("bare", CalendarEventKind::Bare, "2026-04-28"),
            assignment("first", Some(0), None),
        ];

        sort_week_events(&mut events);

        assert_eq!(uids(&events), ["first", "bare", "second"]);
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

        sort_week_events(&mut events);

        assert_eq!(events[0].date, "2026-04-27");
        assert_eq!(events[1].date, "2026-04-28");
    }
}
