use chrono::{NaiveDate, NaiveTime};
use std::collections::HashMap;

use super::events::classify_event;
use super::ical::ORDER_PROPERTY;
use super::types::{PendingEvent, RawVEvent};

const WINDOW_START_MINUTE: u32 = 8 * 60;
const WINDOW_LENGTH_MINUTES: u32 = 8 * 60;

pub(super) fn full_window() -> (NaiveTime, NaiveTime) {
    (
        minute_of_day(WINDOW_START_MINUTE),
        minute_of_day(WINDOW_START_MINUTE + WINDOW_LENGTH_MINUTES),
    )
}

/// Entries are `(order index, UID)` and are sorted on the order index with the UID as
/// tie-breaker, so the allocation is canonical regardless of input order and the earliest
/// card in a cell gets the earliest slot.
/// Boundary i sits at start + (i * length) / n minutes, so the first slot starts at
/// 08:00, the last ends at 16:00, and adjacent slots share a boundary without overlap.
pub(super) fn allocate_slots(entries: &[(u32, String)]) -> Vec<(String, NaiveTime, NaiveTime)> {
    if entries.is_empty() {
        return Vec::new();
    }
    let mut sorted = entries.to_vec();
    sorted.sort();
    let n = sorted.len() as u32;
    sorted
        .into_iter()
        .enumerate()
        .map(|(i, (_, uid))| {
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

/// `etag` guards the PUT via If-Match and is empty when the server supplied none.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SlotUpdate {
    pub(super) href: String,
    pub(super) uid: String,
    pub(super) etag: String,
    pub(super) payload: String,
}

/// One assignment singled out of the day, either because the caller is about to write it
/// itself or because it is being moved to a new position within the day.
pub(super) struct DayPlacement<'a> {
    pub(super) uid: &'a str,
    /// Requested position among the day's assignments. `None` keeps the event where it
    /// already sits, or appends it when the day does not contain it yet.
    pub(super) order_index: Option<u32>,
    /// True when the caller PUTs this event itself, so the plan must not update it as well.
    pub(super) written_by_caller: bool,
}

pub(super) struct DayPlan {
    /// Order index and slot the placed event must be written with.
    pub(super) placed: Option<(u32, NaiveTime, NaiveTime)>,
    pub(super) updates: Vec<SlotUpdate>,
}

/// Only events whose DESCRIPTION first line is a `daylite:` reference take part, so bare,
/// absence, and holiday events are never re-slotted. Events already sitting in their slot
/// are skipped so repeated runs converge without extra writes.
///
/// Excluded events keep whatever times they already have and take no share of the window,
/// so the participating assignments are spread across the full 08:00-16:00 window and may
/// overlap an excluded event. That is intended: the alternative would be to silently shrink
/// everyone else's slot on the guess that the untouched event still occupies its old times.
///
/// The day is re-sequenced to a dense 0..n-1 ordering on every call, so an event is updated
/// whenever its order index or its slot times no longer match.
///
/// A `placement` names an event the caller singles out: it lands at the requested position,
/// and when `written_by_caller` is set the caller PUTs it itself, so no update is planned
/// for it and each event is written exactly once per operation. Its resulting index and
/// slot come back via `DayPlan::placed`.
pub(super) fn plan_slot_updates(
    events: &[RawVEvent],
    date: &str,
    placement: Option<DayPlacement>,
) -> DayPlan {
    let assignments: Vec<_> = events
        .iter()
        .map(|event| (classify_event(event), event))
        .filter(|(pending, event)| {
            pending.date == date
                && pending.project_ref.is_some()
                && !pending.href.is_empty()
                && can_patch_slot(&event.raw_ical)
        })
        .collect();

    let entries = sequence_day(&assignments, placement.as_ref());
    let assigned: HashMap<String, (u32, NaiveTime, NaiveTime)> = allocate_slots(&entries)
        .into_iter()
        .enumerate()
        .map(|(index, (uid, start, end))| (uid, (index as u32, start, end)))
        .collect();

    let written_by_caller = placement
        .as_ref()
        .filter(|placement| placement.written_by_caller)
        .map(|placement| placement.uid);

    let updates = assignments
        .into_iter()
        .filter(|(pending, _)| Some(pending.uid.as_str()) != written_by_caller)
        .filter_map(|(pending, event)| {
            let (index, start, end) = *assigned.get(&pending.uid)?;
            let start_str = start.format("%H:%M").to_string();
            let end_str = end.format("%H:%M").to_string();
            if pending.order_index == Some(index)
                && pending.start_time.as_deref() == Some(start_str.as_str())
                && pending.end_time.as_deref() == Some(end_str.as_str())
            {
                return None;
            }
            let payload = patch_event_slot(&event.raw_ical, date, start, end, index);
            Some(SlotUpdate {
                href: pending.href,
                uid: pending.uid,
                etag: event.etag.clone(),
                payload,
            })
        })
        .collect();

    DayPlan {
        placed: placement.and_then(|placement| assigned.get(placement.uid).copied()),
        updates,
    }
}

/// Ranks the day's assignments by their persisted order index (unindexed ones last, UID as
/// tie-breaker) and returns the dense `(index, uid)` sequence the allocation runs on.
///
/// Existing events take odd sort keys so a requested position `d` (key `2d`) lands ahead of
/// whoever currently occupies rank `d` rather than tying with it.
fn sequence_day(
    assignments: &[(PendingEvent, &RawVEvent)],
    placement: Option<&DayPlacement>,
) -> Vec<(u32, String)> {
    let mut ranked: Vec<(u32, &str)> = assignments
        .iter()
        .map(|(pending, _)| {
            (
                pending.order_index.unwrap_or(u32::MAX),
                pending.uid.as_str(),
            )
        })
        .collect();
    ranked.sort();

    let placed_uid = placement.map(|placement| placement.uid);
    let mut current_position = None;
    let mut keys: Vec<(u32, &str)> = Vec::with_capacity(ranked.len() + 1);
    for (_, uid) in ranked {
        if Some(uid) == placed_uid {
            current_position = Some(keys.len() as u32);
            continue;
        }
        keys.push((keys.len() as u32 * 2 + 1, uid));
    }

    if let Some(placement) = placement {
        let position = placement.order_index.or(current_position);
        keys.push((
            position.map_or(u32::MAX, |position| position.saturating_mul(2)),
            placement.uid,
        ));
    }
    keys.sort();

    keys.into_iter()
        .enumerate()
        .map(|(index, (_, uid))| (index as u32, uid.to_string()))
        .collect()
}

/// Every line other than the VEVENT's own DTSTART/DTEND and order property is copied through
/// untouched, so user-added content (extra DESCRIPTION lines, LOCATION, alarms) survives
/// re-slotting and a VTIMEZONE's DTSTART is left alone. A missing DTEND or order property is
/// inserted before END:VEVENT.
fn patch_event_slot(
    raw_ical: &str,
    date: &str,
    start: NaiveTime,
    end: NaiveTime,
    order_index: u32,
) -> String {
    let compact = date.replace('-', "");
    let dtstart = format!("DTSTART:{compact}T{}", start.format("%H%M%S"));
    let dtend = format!("DTEND:{compact}T{}", end.format("%H%M%S"));
    let order = format!("{ORDER_PROPERTY}:{order_index}");

    let mut out: Vec<&str> = Vec::new();
    let mut in_vevent = false;
    let mut wrote_dtend = false;
    let mut wrote_order = false;
    for line in raw_ical.lines() {
        if line == "BEGIN:VEVENT" {
            in_vevent = true;
        } else if line == "END:VEVENT" {
            if in_vevent && !wrote_dtend {
                out.push(&dtend);
            }
            if in_vevent && !wrote_order {
                out.push(&order);
            }
            in_vevent = false;
        } else if in_vevent {
            if line.starts_with("DTSTART:") || line.starts_with("DTSTART;") {
                out.push(&dtstart);
                continue;
            }
            if line.starts_with("DTEND:") || line.starts_with("DTEND;") {
                out.push(&dtend);
                wrote_dtend = true;
                continue;
            }
            if is_property(line, ORDER_PROPERTY) {
                out.push(&order);
                wrote_order = true;
                continue;
            }
        }
        out.push(line);
    }
    out.join("\r\n") + "\r\n"
}

/// True if `patch_event_slot` can safely rewrite this resource's DTSTART/DTEND. It cannot,
/// and the event is therefore excluded from re-allocation rather than corrupt it, when:
/// - a rewritten property inside the VEVENT (DTSTART, DTEND, or the order property) is folded
///   across physical lines (patch replaces only
///   the first line and would orphan the continuation). Folds elsewhere are harmless because
///   patch copies every other line through untouched, so they are deliberately allowed,
/// - the resource holds more than one VEVENT (patch shares its DTEND-insertion state, so a
///   recurrence override would be squashed onto the first component's slot),
/// - the VEVENT expresses its end via DURATION (patch adds a DTEND, which RFC 5545 §3.6.1
///   forbids alongside DURATION),
/// - the VEVENT belongs to a repeating series through RRULE, RDATE, or RECURRENCE-ID.
///   Patching the master moves every occurrence, and only the master's own start date
///   triggers re-allocation, so the later occurrences would never be corrected, or
/// - the event does not start and end on the same day (see `spans_multiple_days`).
fn can_patch_slot(raw_ical: &str) -> bool {
    let mut in_vevent = false;
    let mut vevent_count = 0u32;
    let mut after_rewritten_property = false;
    let mut dtstart = None;
    let mut dtend = None;
    for line in raw_ical.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if after_rewritten_property {
                return false;
            }
            continue;
        }
        after_rewritten_property = false;
        if line == "BEGIN:VEVENT" {
            in_vevent = true;
            vevent_count += 1;
        } else if line == "END:VEVENT" {
            in_vevent = false;
        } else if in_vevent {
            if is_property(line, "DURATION")
                || is_property(line, "RRULE")
                || is_property(line, "RDATE")
                || is_property(line, "RECURRENCE-ID")
            {
                return false;
            }
            if is_property(line, "DTSTART") {
                dtstart = Some(line);
                after_rewritten_property = true;
            } else if is_property(line, "DTEND") {
                dtend = Some(line);
                after_rewritten_property = true;
            } else if is_property(line, ORDER_PROPERTY) {
                after_rewritten_property = true;
            }
        }
    }
    vevent_count <= 1 && !spans_multiple_days(dtstart, dtend)
}

/// True when the event covers more than one day. `patch_event_slot` writes both DTSTART and
/// DTEND with the single day being allocated, which would silently collapse a longer span
/// onto its first day.
fn spans_multiple_days(dtstart: Option<&str>, dtend: Option<&str>) -> bool {
    let (Some(dtstart), Some(dtend)) = (dtstart, dtend) else {
        return false;
    };
    let (Some(start), Some(end)) = (property_date(dtstart), property_date(dtend)) else {
        return false;
    };
    // A DATE-valued DTEND is exclusive (RFC 5545 §3.8.2.2), so a single all-day event ends
    // on the day after it starts; a DATE-TIME DTEND names the last day itself.
    let last_day = if property_value(dtend).is_some_and(|value| value.contains('T')) {
        end
    } else {
        end.pred_opt().unwrap_or(end)
    };
    last_day != start
}

/// True when the line carries the named iCal property, i.e. the name is followed by a
/// parameter (`;`) or the value separator (`:`) rather than being a longer property's prefix.
fn is_property(line: &str, name: &str) -> bool {
    line.strip_prefix(name)
        .is_some_and(|rest| rest.starts_with(':') || rest.starts_with(';'))
}

/// The value of a property line, i.e. everything after the last colon, which separates any
/// parameters from the value.
fn property_value(line: &str) -> Option<&str> {
    line.rsplit_once(':').map(|(_, value)| value)
}

fn property_date(line: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(property_value(line)?.get(..8)?, "%Y%m%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn time(h: u32, m: u32) -> NaiveTime {
        NaiveTime::from_hms_opt(h, m, 0).unwrap()
    }

    fn entries(values: &[(u32, &str)]) -> Vec<(u32, String)> {
        values
            .iter()
            .map(|(index, uid)| (*index, uid.to_string()))
            .collect()
    }

    #[test]
    fn single_assignment_receives_full_window() {
        let slots = allocate_slots(&entries(&[(0, "a")]));

        assert_eq!(slots, vec![("a".to_string(), time(8, 0), time(16, 0))]);
    }

    #[test]
    fn two_assignments_receive_half_windows() {
        let slots = allocate_slots(&entries(&[(0, "a"), (1, "b")]));

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
        let slots = allocate_slots(&entries(&[(0, "a"), (1, "b"), (2, "c")]));

        assert_eq!(
            slots,
            vec![
                ("a".to_string(), time(8, 0), time(10, 40)),
                ("b".to_string(), time(10, 40), time(13, 20)),
                ("c".to_string(), time(13, 20), time(16, 0)),
            ]
        );
    }

    #[test]
    fn reordered_input_produces_identical_output() {
        let forward = allocate_slots(&entries(&[(0, "a"), (1, "b"), (2, "c")]));
        let reversed = allocate_slots(&entries(&[(2, "c"), (0, "a"), (1, "b")]));

        assert_eq!(forward, reversed);
    }

    #[test]
    fn order_index_decides_which_assignment_gets_the_earliest_slot() {
        let slots = allocate_slots(&entries(&[(1, "a"), (0, "b")]));

        assert_eq!(
            slots,
            vec![
                ("b".to_string(), time(8, 0), time(12, 0)),
                ("a".to_string(), time(12, 0), time(16, 0)),
            ]
        );
    }

    #[test]
    fn changing_the_order_index_changes_the_allocated_slot() {
        let before = allocate_slots(&entries(&[(0, "a"), (1, "b")]));
        let after = allocate_slots(&entries(&[(1, "a"), (0, "b")]));

        assert_eq!(before[0].0, "a");
        assert_eq!(after[0].0, "b");
    }

    #[test]
    fn equal_order_indices_fall_back_to_the_canonical_uid() {
        let slots = allocate_slots(&entries(&[(0, "b"), (0, "a")]));

        assert_eq!(slots[0].0, "a");
        assert_eq!(slots[1].0, "b");
    }

    #[test]
    fn slots_are_contiguous_and_span_exactly_the_window() {
        for n in 1..=10usize {
            let input: Vec<(u32, String)> =
                (0..n).map(|i| (i as u32, format!("uid-{i:02}"))).collect();
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

    /// An assignment that predates the order property, so it carries no order index.
    fn assignment_event(uid: &str, date: &str, start: &str, end: &str) -> RawVEvent {
        build_assignment_event(uid, date, start, end, None)
    }

    fn ordered_assignment_event(
        uid: &str,
        date: &str,
        start: &str,
        end: &str,
        order_index: u32,
    ) -> RawVEvent {
        build_assignment_event(uid, date, start, end, Some(order_index))
    }

    fn build_assignment_event(
        uid: &str,
        date: &str,
        start: &str,
        end: &str,
        order_index: Option<u32>,
    ) -> RawVEvent {
        let compact = date.replace('-', "");
        let start_compact = start.replace(':', "");
        let end_compact = end.replace(':', "");
        let order_line = order_index
            .map(|index| format!("{ORDER_PROPERTY}:{index}\r\n"))
            .unwrap_or_default();
        let raw_ical = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:{uid}\r\nDTSTART:{compact}T{start_compact}00\r\nDTEND:{compact}T{end_compact}00\r\nSUMMARY:Projekt {uid}\r\nDESCRIPTION:daylite:/v1/projects/{uid}\r\n{order_line}END:VEVENT\r\nEND:VCALENDAR\r\n"
        );
        RawVEvent {
            uid: uid.to_string(),
            summary: format!("Projekt {uid}"),
            description: format!("daylite:/v1/projects/{uid}"),
            dtstart: date.to_string(),
            start_time: Some(start.to_string()),
            end_time: Some(end.to_string()),
            href: format!("/cal/emp/{uid}.ics"),
            etag: format!("\"etag-{uid}\""),
            raw_ical,
            order_index,
            ..Default::default()
        }
    }

    fn pending_placement(uid: &str, order_index: Option<u32>) -> Option<DayPlacement<'_>> {
        Some(DayPlacement {
            uid,
            order_index,
            written_by_caller: true,
        })
    }

    fn reorder_placement(uid: &str, order_index: u32) -> Option<DayPlacement<'_>> {
        Some(DayPlacement {
            uid,
            order_index: Some(order_index),
            written_by_caller: false,
        })
    }

    #[test]
    fn create_redistributes_day_into_halves() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates.len(), 2);
        assert_eq!(updates[0].uid, "uid-a");
        assert!(updates[0].payload.contains("DTSTART:20260506T080000"));
        assert!(updates[0].payload.contains("DTEND:20260506T120000"));
        assert_eq!(updates[1].uid, "uid-b");
        assert!(updates[1].payload.contains("DTSTART:20260506T120000"));
        assert!(updates[1].payload.contains("DTEND:20260506T160000"));
    }

    #[test]
    fn delete_redistributes_remaining_assignments_into_halves() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "10:40"),
            assignment_event("uid-c", "2026-05-06", "13:20", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates.len(), 2);
        assert!(updates[0].payload.contains("DTSTART:20260506T080000"));
        assert!(updates[0].payload.contains("DTEND:20260506T120000"));
        assert!(updates[1].payload.contains("DTSTART:20260506T120000"));
        assert!(updates[1].payload.contains("DTEND:20260506T160000"));
    }

    #[test]
    fn update_moving_assignment_away_restores_full_window_on_source_day() {
        let events = vec![assignment_event("uid-a", "2026-05-06", "08:00", "12:00")];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates.len(), 1);
        assert!(updates[0].payload.contains("DTSTART:20260506T080000"));
        assert!(updates[0].payload.contains("DTEND:20260506T160000"));
    }

    #[test]
    fn assignments_already_in_their_slot_need_no_update() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "12:00", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "12:00", "16:00", 1),
        ];

        assert!(plan_slot_updates(&events, "2026-05-06", None)
            .updates
            .is_empty());
    }

    #[test]
    fn events_on_other_days_are_ignored() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "16:00", 0),
            ordered_assignment_event("uid-b", "2026-05-07", "08:00", "16:00", 0),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.is_empty(),
            "a lone assignment already owns the full window"
        );
    }

    #[test]
    fn slot_update_carries_href_and_etag_for_the_guarded_put() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates[0].href, "/cal/emp/uid-a.ics");
        assert_eq!(updates[0].etag, "\"etag-uid-a\"");
    }

    #[test]
    fn extra_uid_gets_a_slot_without_an_update_of_its_own() {
        let events = vec![assignment_event("uid-a", "2026-05-06", "08:00", "16:00")];

        let plan = plan_slot_updates(&events, "2026-05-06", pending_placement("uid-b", None));

        assert_eq!(plan.placed, Some((1, time(12, 0), time(16, 0))));
        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.updates[0].uid, "uid-a");
        assert!(plan.updates[0].payload.contains("DTEND:20260506T120000"));
    }

    #[test]
    fn extra_uid_alone_receives_the_full_window() {
        let plan = plan_slot_updates(&[], "2026-05-06", pending_placement("uid-new", None));

        assert_eq!(plan.placed, Some((0, time(8, 0), time(16, 0))));
        assert!(plan.updates.is_empty());
    }

    #[test]
    fn same_day_update_counts_its_own_event_only_once() {
        // The event being updated is still on the server; passing it as extra_uid must
        // not double-count it in the allocation or plan a second PUT for it.
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "12:00", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "12:00", "16:00", 1),
        ];

        let plan = plan_slot_updates(&events, "2026-05-06", pending_placement("uid-b", None));

        assert_eq!(plan.placed, Some((1, time(12, 0), time(16, 0))));
        assert!(plan.updates.is_empty(), "uid-a already sits in its slot");
    }

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

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates.len(), 2);
        assert!(updates.iter().all(|u| u.uid.starts_with("uid-")));
        assert!(updates[0].payload.contains("DTEND:20260506T120000"));
        assert!(updates[1].payload.contains("DTSTART:20260506T120000"));
    }

    #[test]
    fn assignment_without_href_is_left_alone() {
        // Without a CalDAV href there is nothing to PUT; the event must not shift
        // its neighbours' slots either.
        let mut orphan = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        orphan.href = String::new();
        let events = vec![
            orphan,
            ordered_assignment_event("uid-b", "2026-05-06", "08:00", "16:00", 0),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.is_empty(),
            "the only addressable assignment already owns the full window"
        );
    }

    #[test]
    fn can_patch_slot_rejects_unsafe_shapes() {
        let cases: &[(&str, bool, &str)] = &[
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                true,
                "plain DTSTART/DTEND event",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDURATION:PT4H\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "DURATION-based end",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nEND:VEVENT\r\nDURATION:PT4H\r\nEND:VCALENDAR\r\n",
                true,
                "DURATION outside any VEVENT must not count",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:x\r\nRECURRENCE-ID:20260506T080000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "recurrence override (multiple VEVENTs)",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nDTSTART;TZID=Europe/Vienna_long_zone_name_that_wraps:\r\n 20260506T080000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "folded DTSTART would orphan its continuation",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nRRULE:FREQ=WEEKLY;COUNT=10\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "RRULE master would shift every occurrence",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nRDATE:20260513T080000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "RDATE also defines a recurrence set",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nRECURRENCE-ID:20260506T080000\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "lone override instance belongs to a series",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260508T120000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "timed event spanning several days would collapse onto its first day",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART;VALUE=DATE:20260506\r\nDTEND;VALUE=DATE:20260510\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                false,
                "multi-day all-day event would lose its remaining days",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART;VALUE=DATE:20260506\r\nDTEND;VALUE=DATE:20260507\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                true,
                "single all-day event: the exclusive DATE DTEND is the next day, not a span",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                true,
                "missing DTEND is inserted on the allocated day, so no span to lose",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nDESCRIPTION:ein sehr langer Text\r\n der umbrochen wurde\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                true,
                "folded DESCRIPTION inside the VEVENT is copied through untouched",
            ),
            (
                "BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/Vienna\r\nX-LIC-LOCATION:Europe/Vienna\r\n continued\r\nEND:VTIMEZONE\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
                true,
                "fold outside the VEVENT is copied through untouched",
            ),
        ];

        for (raw_ical, expected, label) in cases {
            assert_eq!(can_patch_slot(raw_ical), *expected, "case: {label}");
        }
    }

    #[test]
    fn duration_based_assignment_is_excluded_from_reallocation() {
        let mut duration_event = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        duration_event.raw_ical = duration_event
            .raw_ical
            .replace("DTEND:20260506T160000\r\n", "DURATION:PT8H\r\n");
        let events = vec![
            duration_event,
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-c", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.iter().all(|u| u.uid != "uid-a"),
            "a DURATION-based event must never be re-slotted"
        );
        // Intended: the excluded event keeps its 08:00-16:00 times and takes no share of the
        // window, so the two remaining assignments split the full window and overlap it.
        assert_eq!(
            updates.len(),
            2,
            "only uid-b and uid-c participate in the split, so they get halves instead of thirds"
        );
        assert!(updates[0].payload.contains("DTEND:20260506T120000"));
        assert!(updates[1].payload.contains("DTSTART:20260506T120000"));
    }

    #[test]
    fn multi_vevent_resource_is_excluded_from_reallocation() {
        let mut recurring = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        recurring.raw_ical = recurring.raw_ical.replace(
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
            "END:VEVENT\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nRECURRENCE-ID:20260506T080000\r\nDTSTART:20260513T080000\r\nDTEND:20260513T160000\r\nSUMMARY:Projekt uid-a\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );
        let events = vec![
            recurring,
            ordered_assignment_event("uid-b", "2026-05-06", "08:00", "16:00", 0),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.iter().all(|u| u.uid != "uid-a"),
            "a multi-VEVENT resource must never be re-slotted"
        );
        assert!(
            updates.is_empty(),
            "uid-b is the only re-slottable assignment and already owns the full window"
        );
    }

    #[test]
    fn recurring_assignment_is_excluded_from_reallocation() {
        let mut recurring = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        recurring.raw_ical = recurring
            .raw_ical
            .replace("SUMMARY:", "RRULE:FREQ=WEEKLY;COUNT=10\r\nSUMMARY:");
        let events = vec![
            recurring,
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-c", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.iter().all(|u| u.uid != "uid-a"),
            "re-slotting a series master would move every occurrence"
        );
        assert_eq!(
            updates.len(),
            2,
            "only uid-b and uid-c participate, so they split the window in halves"
        );
    }

    #[test]
    fn multi_day_assignment_is_excluded_from_reallocation() {
        let mut multi_day = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        multi_day.raw_ical = multi_day
            .raw_ical
            .replace("DTEND:20260506T160000", "DTEND:20260508T160000");
        let events = vec![
            multi_day,
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-c", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.iter().all(|u| u.uid != "uid-a"),
            "re-slotting a multi-day event would collapse it onto its first day"
        );
        assert_eq!(
            updates.len(),
            2,
            "only uid-b and uid-c participate, so they split the window in halves"
        );
    }

    #[test]
    fn folded_dtstart_resource_is_excluded_from_reallocation() {
        let mut folded = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        folded.raw_ical = folded.raw_ical.replace(
            "DTSTART:20260506T080000\r\n",
            "DTSTART;TZID=Europe/Very_Long_Timezone_Identifier_That_Wraps:\r\n 20260506T080000\r\n",
        );
        let events = vec![
            folded,
            ordered_assignment_event("uid-b", "2026-05-06", "08:00", "16:00", 0),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.iter().all(|u| u.uid != "uid-a"),
            "a resource with a folded line must never be re-slotted"
        );
        assert!(
            updates.is_empty(),
            "uid-b is the only re-slottable assignment and already owns the full window"
        );
    }

    #[test]
    fn day_is_resequenced_to_dense_order_indices() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates.len(), 2);
        assert!(updates[0].payload.contains("X-LKR-ORDER:0"));
        assert!(updates[1].payload.contains("X-LKR-ORDER:1"));
    }

    #[test]
    fn order_index_rather_than_uid_decides_the_allocated_slot() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "16:00", 1),
            ordered_assignment_event("uid-b", "2026-05-06", "08:00", "16:00", 0),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        let uid_b = updates.iter().find(|u| u.uid == "uid-b").unwrap();
        let uid_a = updates.iter().find(|u| u.uid == "uid-a").unwrap();
        assert!(uid_b.payload.contains("DTSTART:20260506T080000"));
        assert!(uid_b.payload.contains("DTEND:20260506T120000"));
        assert!(uid_a.payload.contains("DTSTART:20260506T120000"));
        assert!(uid_a.payload.contains("DTEND:20260506T160000"));
    }

    #[test]
    fn changing_the_order_index_changes_the_allocated_times() {
        let before = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "12:00", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "12:00", "16:00", 1),
        ];
        let after = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "12:00", 1),
            ordered_assignment_event("uid-b", "2026-05-06", "12:00", "16:00", 0),
        ];

        assert!(plan_slot_updates(&before, "2026-05-06", None)
            .updates
            .is_empty());
        let updates = plan_slot_updates(&after, "2026-05-06", None).updates;
        let uid_a = updates.iter().find(|u| u.uid == "uid-a").unwrap();
        assert!(uid_a.payload.contains("DTSTART:20260506T120000"));
    }

    #[test]
    fn a_new_assignment_without_a_requested_position_is_appended() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "16:00", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "08:00", "16:00", 1),
        ];

        let plan = plan_slot_updates(&events, "2026-05-06", pending_placement("uid-new", None));

        assert_eq!(plan.placed.unwrap().0, 2);
    }

    #[test]
    fn a_pending_write_without_a_requested_position_keeps_its_place() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "10:40", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "10:40", "13:20", 1),
            ordered_assignment_event("uid-c", "2026-05-06", "13:20", "16:00", 2),
        ];

        let plan = plan_slot_updates(&events, "2026-05-06", pending_placement("uid-b", None));

        assert_eq!(plan.placed, Some((1, time(10, 40), time(13, 20))));
        assert!(
            plan.updates.is_empty(),
            "nothing else moved, so nothing else is written"
        );
    }

    #[test]
    fn a_pending_write_lands_at_the_requested_position() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "12:00", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "12:00", "16:00", 1),
        ];

        let plan = plan_slot_updates(&events, "2026-05-06", pending_placement("uid-new", Some(1)));

        assert_eq!(plan.placed.unwrap().0, 1, "lands between uid-a and uid-b");
        let uid_b = plan.updates.iter().find(|u| u.uid == "uid-b").unwrap();
        assert!(uid_b.payload.contains("X-LKR-ORDER:2"));
    }

    #[test]
    fn reordering_within_the_day_rewrites_the_moved_card_and_its_neighbours() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "10:40", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "10:40", "13:20", 1),
            ordered_assignment_event("uid-c", "2026-05-06", "13:20", "16:00", 2),
        ];

        let updates =
            plan_slot_updates(&events, "2026-05-06", reorder_placement("uid-c", 0)).updates;

        let uid_c = updates.iter().find(|u| u.uid == "uid-c").unwrap();
        assert!(uid_c.payload.contains("X-LKR-ORDER:0"));
        assert!(uid_c.payload.contains("DTSTART:20260506T080000"));
        let uid_a = updates.iter().find(|u| u.uid == "uid-a").unwrap();
        assert!(uid_a.payload.contains("X-LKR-ORDER:1"));
        assert!(uid_a.payload.contains("DTSTART:20260506T104000"));
    }

    #[test]
    fn reordering_a_card_onto_its_own_position_writes_nothing() {
        let events = vec![
            ordered_assignment_event("uid-a", "2026-05-06", "08:00", "12:00", 0),
            ordered_assignment_event("uid-b", "2026-05-06", "12:00", "16:00", 1),
        ];

        assert!(
            plan_slot_updates(&events, "2026-05-06", reorder_placement("uid-b", 1))
                .updates
                .is_empty()
        );
    }

    #[test]
    fn assignment_excluded_from_reslotting_keeps_its_order_index_and_times() {
        let mut excluded = ordered_assignment_event("uid-a", "2026-05-06", "09:00", "17:00", 1);
        excluded.raw_ical = excluded
            .raw_ical
            .replace("DTEND:20260506T170000\r\n", "DURATION:PT8H\r\n");
        let events = vec![
            excluded,
            ordered_assignment_event("uid-b", "2026-05-06", "08:00", "16:00", 0),
            ordered_assignment_event("uid-c", "2026-05-06", "08:00", "16:00", 2),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.iter().all(|u| u.uid != "uid-a"),
            "an excluded assignment keeps both its order index and its times"
        );
        let uid_c = updates.iter().find(|u| u.uid == "uid-c").unwrap();
        assert!(
            uid_c.payload.contains("X-LKR-ORDER:1"),
            "the participating assignments are re-sequenced around it"
        );
    }

    #[test]
    fn patching_replaces_an_existing_order_property() {
        let raw = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nX-LKR-ORDER:3\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0), 1);

        assert!(patched.contains("X-LKR-ORDER:1"));
        assert!(!patched.contains("X-LKR-ORDER:3"));
    }

    #[test]
    fn patching_inserts_the_order_property_when_missing() {
        let raw = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0), 2);

        assert!(patched.contains("X-LKR-ORDER:2"));
        assert!(patched.contains("END:VEVENT"));
    }

    #[test]
    fn can_patch_slot_rejects_a_folded_order_property() {
        let folded = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\nDTSTART:20260506T080000\r\nDTEND:20260506T120000\r\nX-LKR-ORDER:1\r\n 2\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        assert!(!can_patch_slot(folded));
    }

    #[test]
    fn patching_preserves_user_added_properties_and_alarms() {
        let raw = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nSUMMARY:Projekt Nord\r\nDESCRIPTION:daylite:/v1/projects/42\\nNotiz vom Nutzer\r\nLOCATION:Baustelle Nord\r\nBEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nEND:VALARM\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0), 0);

        assert!(patched.contains("DTSTART:20260506T080000"));
        assert!(patched.contains("DTEND:20260506T120000"));
        assert!(
            patched.contains("DESCRIPTION:daylite:/v1/projects/42\\nNotiz vom Nutzer"),
            "user-added description lines must survive, got: {patched}"
        );
        assert!(patched.contains("LOCATION:Baustelle Nord"));
        assert!(patched.contains("BEGIN:VALARM"));
        assert!(patched.contains("TRIGGER:-PT15M"));
    }

    #[test]
    fn patching_replaces_dtstart_with_parameters() {
        let raw = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART;TZID=Europe/Vienna:20260506T090000\r\nDTEND;TZID=Europe/Vienna:20260506T170000\r\nSUMMARY:Projekt\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(12, 0), time(16, 0), 1);

        assert!(patched.contains("DTSTART:20260506T120000"));
        assert!(patched.contains("DTEND:20260506T160000"));
        assert!(
            !patched.contains("TZID"),
            "old timed properties must be gone"
        );
    }

    #[test]
    fn patching_inserts_dtend_when_missing() {
        let raw = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART;VALUE=DATE:20260506\r\nSUMMARY:Projekt\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0), 0);

        assert!(patched.contains("DTSTART:20260506T080000"));
        assert!(patched.contains("DTEND:20260506T120000"));
    }

    #[test]
    fn patching_leaves_vtimezone_dtstart_untouched() {
        let raw = "BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/Vienna\r\nBEGIN:STANDARD\r\nDTSTART:19701025T030000\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nSUMMARY:Projekt\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0), 0);

        assert!(
            patched.contains("DTSTART:19701025T030000"),
            "VTIMEZONE transition rules must not be rewritten"
        );
        assert!(patched.contains("DTSTART:20260506T080000"));
        assert!(patched.contains("DTEND:20260506T120000"));
    }
}
