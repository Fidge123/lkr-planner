use super::super::types::{PendingEvent, RawVEvent};

const DAYLITE_DESCRIPTION_PREFIX: &str = "daylite:";

pub(crate) fn parse_daylite_reference(description: &str) -> Option<String> {
    // Strip ASCII whitespace, BOM (U+FEFF), and zero-width space (U+200B) that some
    // calendar UIs prepend to the description field.
    let first_line = description
        .lines()
        .next()
        .unwrap_or("")
        .trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}' || c == '\u{200b}');

    let raw_ref = first_line.strip_prefix(DAYLITE_DESCRIPTION_PREFIX)?.trim();
    if raw_ref.is_empty() {
        None
    } else {
        Some(raw_ref.to_string())
    }
}

pub(crate) fn classify_event(event: &RawVEvent) -> PendingEvent {
    let date = event.dtstart.clone();

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
        event.uid.clone()
    };

    let project_ref = parse_daylite_reference(&event.description);

    PendingEvent {
        uid,
        date,
        summary: event.summary.clone(),
        project_ref,
        start_time: event.start_time.clone(),
        end_time: event.end_time.clone(),
        href: event.href.clone(),
        order_index: event.order_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(description: &str) -> RawVEvent {
        RawVEvent {
            uid: "uid-1".to_string(),
            summary: "Projekt Nord".to_string(),
            description: description.to_string(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        }
    }

    #[test]
    fn parses_the_daylite_reference_out_of_a_bare_description() {
        let cases: &[(&str, Option<&str>, &str)] = &[
            (
                "daylite:/v1/projects/3001",
                Some("/v1/projects/3001"),
                "plain daylite reference",
            ),
            (
                "daylite:/v1/projects/4001\nZusätzliche Notizen hier",
                Some("/v1/projects/4001"),
                "only the first line is read",
            ),
            ("Bitte Auto abholen", None, "unrelated description is bare"),
            ("", None, "empty description is bare"),
        ];

        for (description, expected, label) in cases {
            assert_eq!(
                parse_daylite_reference(description).as_deref(),
                *expected,
                "case: {label}"
            );
        }
    }

    #[test]
    fn project_ref_is_read_from_the_first_description_line() {
        let cases: &[(&str, Option<&str>, &str)] = &[
            (
                "daylite:/v1/projects/3001",
                Some("/v1/projects/3001"),
                "plain daylite reference",
            ),
            (
                "daylite:/v1/projects/4001\nZusätzliche Notizen hier",
                Some("/v1/projects/4001"),
                "only the first line is read",
            ),
            (
                "\u{feff}daylite:/v1/projects/5001",
                Some("/v1/projects/5001"),
                "BOM prefix is stripped",
            ),
            ("Bitte Auto abholen", None, "unrelated description is bare"),
            ("", None, "empty description is bare"),
            ("daylite:", None, "reference without a path is bare"),
        ];

        for (description, expected, label) in cases {
            let pending = classify_event(&event(description));

            assert_eq!(pending.project_ref.as_deref(), *expected, "case: {label}");
        }
    }

    #[test]
    fn carries_summary_and_date_through_unchanged() {
        let pending = classify_event(&event("daylite:/v1/projects/3001"));

        assert_eq!(pending.date, "2026-01-26");
        assert_eq!(pending.summary, "Projekt Nord");
    }

    /// The order index is what the reordering feature persists, so classification
    /// must carry it through untouched.
    #[test]
    fn carries_the_order_index_through_classification() {
        let pending = classify_event(&RawVEvent {
            order_index: Some(2),
            ..event("daylite:/v1/projects/3001")
        });

        assert_eq!(pending.order_index, Some(2));
    }

    #[test]
    fn synthesises_uid_for_event_without_uid() {
        let pending = classify_event(&RawVEvent {
            uid: String::new(),
            summary: "Ohne UID".to_string(),
            dtstart: "2026-01-26".to_string(),
            ..Default::default()
        });

        assert!(pending.uid.starts_with("synthetic-"));
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

        let pending = classify_event(&event);

        assert!(!pending.uid.contains('\n'), "UID must not contain newline");
        assert!(!pending.uid.contains('/'), "UID must not contain slash");
    }
}
