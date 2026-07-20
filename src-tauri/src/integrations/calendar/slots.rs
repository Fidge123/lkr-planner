use chrono::NaiveTime;
use std::collections::HashMap;

use super::events::classify_event;
use super::types::RawVEvent;

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

/// A CalDAV PUT needed to move an assignment event into its allocated slot.
/// `payload` is the ready-to-send body; `etag` guards the PUT via If-Match.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct SlotUpdate {
    pub(super) href: String,
    pub(super) uid: String,
    pub(super) etag: String,
    pub(super) payload: String,
}

/// The result of planning a day: the slot reserved for `extra_uid` (if one was given)
/// and the PUTs that move the day's other assignments into their slots.
pub(super) struct DayPlan {
    pub(super) extra_slot: Option<(NaiveTime, NaiveTime)>,
    pub(super) updates: Vec<SlotUpdate>,
}

/// Plans the PUTs that move a day's lkr-planner assignments into their allocated slots.
/// Only events on `date` whose DESCRIPTION first line is a `daylite:` reference take part;
/// bare, absence, and holiday events are never re-slotted. Events already sitting in
/// their slot are skipped so repeated runs converge without extra writes. Events that
/// `patch_event_slot` cannot safely rewrite are excluded entirely rather than risk
/// producing invalid iCal — see `can_patch_slot`.
///
/// `extra_uid` names an event the caller writes itself (a create or update in flight):
/// it participates in the allocation and gets its slot back via `extra_slot`, but no
/// update is planned for it, so each event is written exactly once per operation.
pub(super) fn plan_slot_updates(
    events: &[RawVEvent],
    date: &str,
    extra_uid: Option<&str>,
) -> DayPlan {
    let assignments: Vec<_> = events
        .iter()
        .map(|event| (classify_event(event.clone()), event))
        .filter(|(pending, event)| {
            pending.date == date
                && pending.project_ref.is_some()
                && !pending.href.is_empty()
                && Some(pending.uid.as_str()) != extra_uid
                && can_patch_slot(&event.raw_ical)
        })
        .collect();

    let mut uids: Vec<String> = assignments
        .iter()
        .map(|(pending, _)| pending.uid.clone())
        .collect();
    if let Some(extra) = extra_uid {
        uids.push(extra.to_string());
    }
    let slots: HashMap<String, (NaiveTime, NaiveTime)> = allocate_slots(&uids)
        .into_iter()
        .map(|(uid, start, end)| (uid, (start, end)))
        .collect();

    let updates = assignments
        .into_iter()
        .filter_map(|(pending, event)| {
            let (start, end) = *slots.get(&pending.uid)?;
            let start_str = start.format("%H:%M").to_string();
            let end_str = end.format("%H:%M").to_string();
            if pending.start_time.as_deref() == Some(start_str.as_str())
                && pending.end_time.as_deref() == Some(end_str.as_str())
            {
                return None;
            }
            let payload = patch_event_slot(&event.raw_ical, date, start, end);
            Some(SlotUpdate {
                href: pending.href,
                uid: pending.uid,
                etag: event.etag.clone(),
                payload,
            })
        })
        .collect();

    DayPlan {
        extra_slot: extra_uid.and_then(|uid| slots.get(uid).copied()),
        updates,
    }
}

/// Rewrites the VEVENT's DTSTART and DTEND to the allocated slot while leaving every
/// other line untouched, so user-added content (extra DESCRIPTION lines, LOCATION,
/// alarms) survives re-slotting. Only lines inside BEGIN:VEVENT..END:VEVENT are
/// replaced; a VTIMEZONE's DTSTART lines are left alone. A missing DTEND is inserted
/// before END:VEVENT.
fn patch_event_slot(raw_ical: &str, date: &str, start: NaiveTime, end: NaiveTime) -> String {
    let compact = date.replace('-', "");
    let dtstart = format!("DTSTART:{compact}T{}", start.format("%H%M%S"));
    let dtend = format!("DTEND:{compact}T{}", end.format("%H%M%S"));

    let mut out: Vec<&str> = Vec::new();
    let mut in_vevent = false;
    let mut wrote_dtend = false;
    for line in raw_ical.lines() {
        if line == "BEGIN:VEVENT" {
            in_vevent = true;
        } else if line == "END:VEVENT" {
            if in_vevent && !wrote_dtend {
                out.push(&dtend);
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
        }
        out.push(line);
    }
    out.join("\r\n") + "\r\n"
}

/// True if `patch_event_slot` can safely rewrite this resource's DTSTART/DTEND. It cannot,
/// and the event is therefore excluded from re-allocation rather than risk invalid iCal, when:
/// - a line is an RFC 5545 folded continuation (patch works on physical lines and would
///   orphan it),
/// - the resource holds more than one VEVENT (patch shares its DTEND-insertion state, so a
///   recurrence override would be squashed onto the first component's slot), or
/// - the VEVENT expresses its end via DURATION (patch adds a DTEND, which RFC 5545 §3.6.1
///   forbids alongside DURATION).
fn can_patch_slot(raw_ical: &str) -> bool {
    let mut in_vevent = false;
    let mut vevent_count = 0u32;
    for line in raw_ical.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            return false;
        }
        if line == "BEGIN:VEVENT" {
            in_vevent = true;
            vevent_count += 1;
        } else if line == "END:VEVENT" {
            in_vevent = false;
        } else if in_vevent && (line.starts_with("DURATION:") || line.starts_with("DURATION;")) {
            return false;
        }
    }
    vevent_count <= 1
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

    #[test]
    fn reordered_input_produces_identical_output() {
        let forward = allocate_slots(&uids(&["a", "b", "c"]));
        let reversed = allocate_slots(&uids(&["c", "a", "b"]));

        assert_eq!(forward, reversed);
    }

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

    fn assignment_event(uid: &str, date: &str, start: &str, end: &str) -> RawVEvent {
        let compact = date.replace('-', "");
        let start_compact = start.replace(':', "");
        let end_compact = end.replace(':', "");
        let raw_ical = format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:{uid}\r\nDTSTART:{compact}T{start_compact}00\r\nDTEND:{compact}T{end_compact}00\r\nSUMMARY:Projekt {uid}\r\nDESCRIPTION:daylite:/v1/projects/{uid}\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
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
        // A day previously split into thirds after one of three assignments was deleted.
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
        // Source day after the moved event left: one half-window assignment remains.
        let events = vec![assignment_event("uid-a", "2026-05-06", "08:00", "12:00")];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert_eq!(updates.len(), 1);
        assert!(updates[0].payload.contains("DTSTART:20260506T080000"));
        assert!(updates[0].payload.contains("DTEND:20260506T160000"));
    }

    #[test]
    fn assignments_already_in_their_slot_need_no_update() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "12:00"),
            assignment_event("uid-b", "2026-05-06", "12:00", "16:00"),
        ];

        assert!(plan_slot_updates(&events, "2026-05-06", None)
            .updates
            .is_empty());
    }

    #[test]
    fn events_on_other_days_are_ignored() {
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "16:00"),
            assignment_event("uid-b", "2026-05-07", "08:00", "16:00"),
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
        // One existing assignment; a create in flight for uid-b: uid-b sorts after
        // uid-a, so it receives the afternoon half while uid-a is moved to the morning.
        let events = vec![assignment_event("uid-a", "2026-05-06", "08:00", "16:00")];

        let plan = plan_slot_updates(&events, "2026-05-06", Some("uid-b"));

        assert_eq!(plan.extra_slot, Some((time(12, 0), time(16, 0))));
        assert_eq!(plan.updates.len(), 1);
        assert_eq!(plan.updates[0].uid, "uid-a");
        assert!(plan.updates[0].payload.contains("DTEND:20260506T120000"));
    }

    #[test]
    fn extra_uid_alone_receives_the_full_window() {
        let plan = plan_slot_updates(&[], "2026-05-06", Some("uid-new"));

        assert_eq!(plan.extra_slot, Some((time(8, 0), time(16, 0))));
        assert!(plan.updates.is_empty());
    }

    #[test]
    fn same_day_update_counts_its_own_event_only_once() {
        // The event being updated is still on the server; passing it as extra_uid must
        // not double-count it in the allocation or plan a second PUT for it.
        let events = vec![
            assignment_event("uid-a", "2026-05-06", "08:00", "12:00"),
            assignment_event("uid-b", "2026-05-06", "12:00", "16:00"),
        ];

        let plan = plan_slot_updates(&events, "2026-05-06", Some("uid-b"));

        assert_eq!(plan.extra_slot, Some((time(12, 0), time(16, 0))));
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

        // Only the two daylite assignments are re-slotted; they split the window in halves.
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
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
        ];

        let updates = plan_slot_updates(&events, "2026-05-06", None).updates;

        assert!(
            updates.is_empty(),
            "the only addressable assignment already owns the full window"
        );
    }

    // ── Unsafe-to-patch event shapes ──

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
                "folded continuation line",
            ),
        ];

        for (raw_ical, expected, label) in cases {
            assert_eq!(can_patch_slot(raw_ical), *expected, "case: {label}");
        }
    }

    #[test]
    fn duration_based_assignment_is_excluded_from_reallocation() {
        // Simulates an event whose end was edited to DURATION in an external calendar
        // client. Patching it would insert a DTEND alongside the existing DURATION,
        // producing an invalid VEVENT (RFC 5545 §3.6.1), so it must be skipped entirely.
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
        // Simulates a recurrence override added in an external calendar client:
        // patching it would squash both VEVENTs onto the same slot.
        let mut recurring = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        recurring.raw_ical = recurring.raw_ical.replace(
            "END:VEVENT\r\nEND:VCALENDAR\r\n",
            "END:VEVENT\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nRECURRENCE-ID:20260506T080000\r\nDTSTART:20260513T080000\r\nDTEND:20260513T160000\r\nSUMMARY:Projekt uid-a\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n",
        );
        let events = vec![
            recurring,
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
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
    fn folded_dtstart_resource_is_excluded_from_reallocation() {
        // Simulates a long TZID parameter folded onto a continuation line.
        let mut folded = assignment_event("uid-a", "2026-05-06", "08:00", "16:00");
        folded.raw_ical = folded.raw_ical.replace(
            "DTSTART:20260506T080000\r\n",
            "DTSTART;TZID=Europe/Very_Long_Timezone_Identifier_That_Wraps:\r\n 20260506T080000\r\n",
        );
        let events = vec![
            folded,
            assignment_event("uid-b", "2026-05-06", "08:00", "16:00"),
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
    fn patching_preserves_user_added_properties_and_alarms() {
        let raw = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nSUMMARY:Projekt Nord\r\nDESCRIPTION:daylite:/v1/projects/42\\nNotiz vom Nutzer\r\nLOCATION:Baustelle Nord\r\nBEGIN:VALARM\r\nTRIGGER:-PT15M\r\nACTION:DISPLAY\r\nEND:VALARM\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0));

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

        let patched = patch_event_slot(raw, "2026-05-06", time(12, 0), time(16, 0));

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

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0));

        assert!(patched.contains("DTSTART:20260506T080000"));
        assert!(patched.contains("DTEND:20260506T120000"));
    }

    #[test]
    fn patching_leaves_vtimezone_dtstart_untouched() {
        let raw = "BEGIN:VCALENDAR\r\nBEGIN:VTIMEZONE\r\nTZID:Europe/Vienna\r\nBEGIN:STANDARD\r\nDTSTART:19701025T030000\r\nEND:STANDARD\r\nEND:VTIMEZONE\r\nBEGIN:VEVENT\r\nUID:uid-a\r\nDTSTART:20260506T080000\r\nDTEND:20260506T160000\r\nSUMMARY:Projekt\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

        let patched = patch_event_slot(raw, "2026-05-06", time(8, 0), time(12, 0));

        assert!(
            patched.contains("DTSTART:19701025T030000"),
            "VTIMEZONE transition rules must not be rewritten"
        );
        assert!(patched.contains("DTSTART:20260506T080000"));
        assert!(patched.contains("DTEND:20260506T120000"));
    }
}
