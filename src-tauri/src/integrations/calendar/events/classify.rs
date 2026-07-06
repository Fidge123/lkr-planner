use super::super::types::{PendingEvent, RawVEvent};

const DAYLITE_DESCRIPTION_PREFIX: &str = "daylite:";

/// Classifies a raw VEVENT as a lkr-planner assignment or a bare calendar event.
pub(crate) fn classify_event(event: RawVEvent) -> PendingEvent {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
