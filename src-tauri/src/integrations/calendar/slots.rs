use chrono::NaiveTime;
use std::collections::HashMap;

use super::events::classify_event;
use super::types::RawVEvent;

// ── Deterministic slot allocation ─────────────────────────────────────────────

/// Fixed working window: 08:00-16:00 floating local time, 480 minutes.
const WINDOW_START_MINUTE: u32 = 8 * 60;
const WINDOW_LENGTH_MINUTES: u32 = 8 * 60;

/// Returns the full 08:00-16:00 window as (start, end).
pub(super) fn full_window() -> (NaiveTime, NaiveTime) {
    (
        minute_of_day(WINDOW_START_MINUTE),
        minute_of_day(WINDOW_START_MINUTE + WINDOW_LENGTH_MINUTES),
    )
}

/// Splits the fixed window into one non-overlapping [start, end) slot per UID.
/// UIDs are sorted first so the allocation is canonical regardless of input order.
/// Boundary i sits at start + (i * length) / n minutes, so the first slot starts at
/// 08:00, the last ends at 16:00, and adjacent slots share a boundary without overlap.
pub(super) fn allocate_slots(uids: &[String]) -> Vec<(String, NaiveTime, NaiveTime)> {
    if uids.is_empty() {
        return Vec::new();
    }
    let mut sorted = uids.to_vec();
    sorted.sort();
    let n = sorted.len() as u32;
    sorted
        .into_iter()
        .enumerate()
        .map(|(i, uid)| {
            let i = i as u32;
            let start = WINDOW_START_MINUTE + (i * WINDOW_LENGTH_MINUTES) / n;
            let end = WINDOW_START_MINUTE + ((i + 1) * WINDOW_LENGTH_MINUTES) / n;
            (uid, minute_of_day(start), minute_of_day(end))
        })
        .collect()
}

fn minute_of_day(minute: u32) -> NaiveTime {
    NaiveTime::from_num_seconds_from_midnight_opt(minute * 60, 0)
        .expect("minute within the 08:00-16:00 window is a valid time of day")
}

// ── Re-allocation planning ────────────────────────────────────────────────────

/// A CalDAV PUT needed to move an assignment event into its allocated slot.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SlotUpdate {
    pub(super) href: String,
    pub(super) uid: String,
    pub(super) summary: String,
    pub(super) project_ref: String,
    pub(super) start: NaiveTime,
    pub(super) end: NaiveTime,
}

/// Plans the PUTs that move a day's lkr-planner assignments into their allocated slots.
/// Only events on `date` whose DESCRIPTION first line is a `daylite:` reference take part;
/// bare, absence, and holiday events are never re-slotted. Events already sitting in
/// their slot are skipped so repeated runs converge without extra writes.
pub(super) fn plan_slot_updates(events: &[RawVEvent], date: &str) -> Vec<SlotUpdate> {
    let assignments: Vec<_> = events
        .iter()
        .cloned()
        .map(classify_event)
        .filter(|p| p.date == date && p.project_ref.is_some() && !p.href.is_empty())
        .collect();

    let uids: Vec<String> = assignments.iter().map(|p| p.uid.clone()).collect();
    let slots: HashMap<String, (NaiveTime, NaiveTime)> = allocate_slots(&uids)
        .into_iter()
        .map(|(uid, start, end)| (uid, (start, end)))
        .collect();

    assignments
        .into_iter()
        .filter_map(|pending| {
            let (start, end) = *slots.get(&pending.uid)?;
            let start_str = start.format("%H:%M").to_string();
            let end_str = end.format("%H:%M").to_string();
            if pending.start_time.as_deref() == Some(start_str.as_str())
                && pending.end_time.as_deref() == Some(end_str.as_str())
            {
                return None;
            }
            Some(SlotUpdate {
                href: pending.href,
                uid: pending.uid,
                summary: pending.summary,
                project_ref: pending
                    .project_ref
                    .expect("filtered to assignments with a project_ref"),
                start,
                end,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn uids(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    // ── allocate_slots: window splitting ──

    #[test]
    fn single_assignment_receives_full_window() {
        let slots = allocate_slots(&uids(&["a"]));

        assert_eq!(slots, vec![("a".to_string(), time(8, 0), time(16, 0))]);
    }

    #[test]
    fn two_assignments_receive_half_windows() {
        let slots = allocate_slots(&uids(&["a", "b"]));

        assert_eq!(
            slots,
            vec![
                ("a".to_string(), time(8, 0), time(12, 0)),
                ("b".to_string(), time(12, 0), time(16, 0)),
            ]
        );
    }

    #[test]
    fn three_assignments_receive_third_windows_at_minute_granularity() {
        let slots = allocate_slots(&uids(&["a", "b", "c"]));

        assert_eq!(
            slots,
            vec![
                ("a".to_string(), time(8, 0), time(10, 40)),
                ("b".to_string(), time(10, 40), time(13, 20)),
                ("c".to_string(), time(13, 20), time(16, 0)),
            ]
        );
    }

    // ── allocate_slots: determinism ──

    #[test]
    fn reordered_input_produces_identical_output() {
        let forward = allocate_slots(&uids(&["a", "b", "c"]));
        let reversed = allocate_slots(&uids(&["c", "a", "b"]));

        assert_eq!(forward, reversed);
    }

    // ── allocate_slots: boundaries and edge cases ──

    #[test]
    fn slots_are_contiguous_and_span_exactly_the_window() {
        for n in 1..=10usize {
            let input: Vec<String> = (0..n).map(|i| format!("uid-{i:02}")).collect();
            let slots = allocate_slots(&input);

            assert_eq!(slots.len(), n);
            assert_eq!(
                slots.first().unwrap().1,
                time(8, 0),
                "first slot starts at 08:00"
            );
            assert_eq!(
                slots.last().unwrap().2,
                time(16, 0),
                "last slot ends at 16:00"
            );
            for pair in slots.windows(2) {
                assert_eq!(
                    pair[0].2, pair[1].1,
                    "slot end must equal the next slot's start (n={n})"
                );
                assert!(
                    pair[0].1 < pair[0].2,
                    "slots must have positive length (n={n})"
                );
            }
        }
    }

    #[test]
    fn empty_input_returns_empty_allocation() {
        assert!(allocate_slots(&[]).is_empty());
    }

    // ── plan_slot_updates: write scenarios ──

    fn assignment_event(uid: &str, date: &str, start: &str, end: &str) -> RawVEvent {
        RawVEvent {
            uid: uid.to_string(),
            summary: format!("Projekt {uid}"),
            description: format!("daylite:/v1/projects/{uid}"),
            dtstart: date.to_string(),
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            href: format!("/cal/emp/{uid}.ics"),
            ..Default::default()
        }
    }

    #[test]
    fn create_redistributes_day_into_halves() {
        // An existing full-window assignment plus a freshly created one (also full window).
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06");

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].uid, "uid-a");
        assert_eq!(
            (updates[0].start, updates[0].end),
            (time(8, 0), time(12, 0))
        );
        assert_eq!(updates[1].uid, "uid-b");
        assert_eq!(
            (updates[1].start, updates[1].end),
            (time(12, 0), time(16, 0))
        );
    }

    #[test]
    fn delete_redistributes_remaining_assignments_into_halves() {
        // A day previously split into thirds after one of three assignments was deleted.
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "10:40"),
            assignment_event("uid-c", "2026-05-06", "13:20", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06");

        assert_eq!(updates.len(), 2);
        assert_eq!(
            (updates[0].start, updates[0].end),
            (time(8, 0), time(12, 0))
        );
        assert_eq!(
            (updates[1].start, updates[1].end),
            (time(12, 0), time(16, 0))
        );
    }

    #[test]
    fn update_moving_assignment_away_restores_full_window_on_source_day() {
        // Source day after the moved event left: one half-window assignment remains.
        let events = vec![assignment_event("uid-a", "2026-05-06", "08:00", "12:00")];

        let updates = plan_slot_updates(&events, "2026-05-06");

        assert_eq!(updates.len(), 1);
        assert_eq!(
            (updates[0].start, updates[0].end),
            (time(8, 0), time(16, 0))
        );
    }

    #[test]
    fn assignments_already_in_their_slot_need_no_update() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "12:00"),
            assignment_event("uid-b", "2026-05-06", "12:00", "16:00"),
        ];

        assert!(plan_slot_updates(&events, "2026-05-06").is_empty());
    }

    #[test]
    fn events_on_other_days_are_ignored() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-07", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06");

        assert!(
            updates.is_empty(),
            "a lone assignment already owns the full window"
        );
    }

    #[test]
    fn slot_update_carries_summary_and_project_ref_for_the_rewrite() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06");

        assert_eq!(updates[0].summary, "Projekt uid-a");
        assert_eq!(updates[0].project_ref, "/v1/projects/uid-a");
        assert_eq!(updates[0].href, "/cal/emp/uid-a.ics");
    }

    // ── plan_slot_updates: exclusion of non-assignment events ──

    #[test]
    fn bare_absence_and_holiday_events_are_never_reslotted() {
        let bare = RawVEvent {
            uid: "bare-1".to_string(),
            summary: "Auto Werkstatt".to_string(),
            description: "Bitte Auto abholen".to_string(),
            dtstart: "2026-05-06".to_string(),
            start_time: Some("08:00".to_string()),
            end_time: Some("16:00".to_string()),
            href: "/cal/emp/bare-1.ics".to_string(),
            ..Default::default()
        };
        let all_day_absence = RawVEvent {
            uid: "abs-1".to_string(),
            summary: "Urlaub".to_string(),
            description: String::new(),
            dtstart: "2026-05-06".to_string(),
            href: "/cal/emp/abs-1.ics".to_string(),
            ..Default::default()
        };
        let holiday = RawVEvent {
            uid: "holiday-1".to_string(),
            summary: "Tag der Arbeit".to_string(),
            description: String::new(),
            dtstart: "2026-05-06".to_string(),
            href: "/cal/emp/holiday-1.ics".to_string(),
            ..Default::default()
        };
        let events = vec![
            bare,
            all_day_absence,
            holiday,
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06");

        // Only the two daylite assignments are re-slotted; they split the window in halves.
        assert_eq!(updates.len(), 2);
        assert!(updates.iter().all(|u| u.uid.starts_with("uid-")));
        assert_eq!(
            (updates[0].start, updates[0].end),
            (time(8, 0), time(12, 0))
        );
        assert_eq!(
            (updates[1].start, updates[1].end),
            (time(12, 0), time(16, 0))
        );
    }

    #[test]
    fn assignment_without_href_is_left_alone() {
        // Without a CalDAV href there is nothing to PUT; the event must not shift
        // its neighbours' slots either.
        let mut orphan = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        orphan.href = String::new();
        let events = vec![
            orphan,
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06");

        assert!(
            updates.is_empty(),
            "the only addressable assignment already owns the full window"
        );
    }
}
