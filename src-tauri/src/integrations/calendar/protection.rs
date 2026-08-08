use super::caldav::{fetch_event_by_href, CaldavSession};
use super::events::parse_daylite_reference;
use crate::integrations::daylite::projects::{
    fetch_project_by_reference, ResolvedProject, FIXED_APPOINTMENT_CATEGORY,
};

pub(crate) const FIXED_APPOINTMENT_MESSAGE: &str = "Dieser Termin ist als 'Termin FIX geplant' gesperrt und kann nicht geändert, auf einen anderen Tag verschoben oder gelöscht werden.";

/// Refuses a write to an event whose Daylite project is a fixed appointment.
/// The link comes from the event, never the caller: an override decides only whether to proceed, not what counts as protected.
pub(crate) async fn refuse_protected_event(
    app: tauri::AppHandle,
    session: &CaldavSession,
    href: &str,
    override_protection: bool,
) -> Result<(), String> {
    // An overridden write skips the lookups too, so it is cheaper than a checked one.
    if override_protection {
        return Ok(());
    }

    // A missing event cannot be protected; delete stays idempotent for one.
    let Some(event) = fetch_event_by_href(session, href).await? else {
        return Ok(());
    };

    let (project_ref, project) = resolve_project_link(app, href, &event.description).await;
    refuse(is_protected_event(project_ref.as_deref(), project.as_ref()))
}

/// Only the day is committed, so a move that keeps the date is allowed.
pub(crate) async fn refuse_protected_day_change(
    app: tauri::AppHandle,
    session: &CaldavSession,
    href: &str,
    target_date: &str,
) -> Result<(), String> {
    let Some(event) = fetch_event_by_href(session, href).await? else {
        return Ok(());
    };

    let (project_ref, project) = resolve_project_link(app, href, &event.description).await;
    refuse(protects_day_change(
        &event.dtstart,
        target_date,
        project_ref.as_deref(),
        project.as_ref(),
    ))
}

async fn resolve_project_link(
    app: tauri::AppHandle,
    href: &str,
    description: &str,
) -> (Option<String>, Option<ResolvedProject>) {
    let project_ref = parse_daylite_reference(description);
    let project = match project_ref.as_deref() {
        Some(reference) => fetch_project_by_reference(app, reference).await,
        None => None,
    };
    if project_ref.is_some() && project.is_none() {
        eprintln!("calendar: project lookup failed for {href}, treating the event as unprotected");
    }
    (project_ref, project)
}

fn refuse(protected: bool) -> Result<(), String> {
    if protected {
        return Err(FIXED_APPOINTMENT_MESSAGE.to_string());
    }
    Ok(())
}

fn protects_day_change(
    event_date: &str,
    target_date: &str,
    project_ref: Option<&str>,
    project: Option<&ResolvedProject>,
) -> bool {
    event_date != target_date && is_protected_event(project_ref, project)
}

/// A bare event was not created by the planner, so it is protected too.
/// A project that cannot be resolved fails open: a broken link must not lock an event.
fn is_protected_event(project_ref: Option<&str>, project: Option<&ResolvedProject>) -> bool {
    if project_ref.is_none() {
        return true;
    }
    project.and_then(|p| p.category.as_deref()) == Some(FIXED_APPOINTMENT_CATEGORY)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXED_REF: Option<&str> = Some("/v1/projects/3001");

    fn project(category: Option<&str>) -> ResolvedProject {
        ResolvedProject {
            name: "Projekt Nord".to_string(),
            status: "in_progress".to_string(),
            category: category.map(str::to_string),
        }
    }

    #[test]
    fn event_of_a_fixed_appointment_project_is_protected() {
        let resolved = project(Some(FIXED_APPOINTMENT_CATEGORY));

        assert!(is_protected_event(FIXED_REF, Some(&resolved)));
    }

    #[test]
    fn event_of_another_project_category_is_not_protected() {
        for category in [Some("Liefertermin bekannt"), None] {
            let resolved = project(category);

            assert!(!is_protected_event(FIXED_REF, Some(&resolved)));
        }
    }

    #[test]
    fn event_without_a_daylite_reference_is_protected() {
        assert!(is_protected_event(None, None));
    }

    #[test]
    fn event_whose_project_lookup_failed_is_not_protected() {
        assert!(!is_protected_event(FIXED_REF, None));
    }

    #[test]
    fn moving_a_fixed_appointment_to_another_day_is_refused() {
        let resolved = project(Some(FIXED_APPOINTMENT_CATEGORY));

        assert!(protects_day_change(
            "2026-05-06",
            "2026-05-07",
            FIXED_REF,
            Some(&resolved)
        ));
    }

    #[test]
    fn moving_a_fixed_appointment_within_its_day_is_allowed() {
        let resolved = project(Some(FIXED_APPOINTMENT_CATEGORY));

        assert!(!protects_day_change(
            "2026-05-06",
            "2026-05-06",
            FIXED_REF,
            Some(&resolved)
        ));
    }

    #[test]
    fn moving_an_unprotected_assignment_to_another_day_is_allowed() {
        let resolved = project(Some("Liefertermin bekannt"));

        assert!(!protects_day_change(
            "2026-05-06",
            "2026-05-07",
            FIXED_REF,
            Some(&resolved)
        ));
    }
}
