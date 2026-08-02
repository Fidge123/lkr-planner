use super::caldav::{fetch_event_description, CaldavSession};
use super::events::parse_daylite_reference;
use crate::integrations::daylite::projects::{
    fetch_project_by_reference, ResolvedProject, FIXED_APPOINTMENT_CATEGORY,
};

pub(crate) const FIXED_APPOINTMENT_MESSAGE: &str =
    "Dieser Termin ist als 'Termin FIX geplant' gesperrt und kann nicht geändert oder gelöscht werden.";

/// Rejects the write before it is issued when the event behind `href` belongs to a
/// Daylite project marked as a fixed appointment. The project link is read from the
/// event itself, never from a caller-supplied reference, so the check cannot be
/// bypassed from the frontend.
pub(crate) async fn refuse_protected_event(
    app: tauri::AppHandle,
    session: &CaldavSession,
    href: &str,
) -> Result<(), String> {
    // A missing event cannot be protected; delete stays idempotent for one.
    let Some(description) = fetch_event_description(session, href).await? else {
        return Ok(());
    };

    let project_ref = parse_daylite_reference(&description);
    let project = match project_ref.as_deref() {
        Some(reference) => fetch_project_by_reference(app, reference).await,
        None => None,
    };
    if project_ref.is_some() && project.is_none() {
        eprintln!("calendar: project lookup failed for {href}, treating the event as unprotected");
    }

    if is_protected_event(project_ref.as_deref(), project.as_ref()) {
        return Err(FIXED_APPOINTMENT_MESSAGE.to_string());
    }
    Ok(())
}

/// An event without a Daylite reference was not created by the planner, so it is
/// protected as well. A project that cannot be resolved fails open, so a broken
/// Daylite link never locks an event permanently.
fn is_protected_event(project_ref: Option<&str>, project: Option<&ResolvedProject>) -> bool {
    if project_ref.is_none() {
        return true;
    }
    project.and_then(|p| p.category.as_deref()) == Some(FIXED_APPOINTMENT_CATEGORY)
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(is_protected_event(
            Some("/v1/projects/3001"),
            Some(&resolved)
        ));
    }

    #[test]
    fn event_of_another_project_category_is_not_protected() {
        for category in [Some("Liefertermin bekannt"), None] {
            let resolved = project(category);

            assert!(!is_protected_event(
                Some("/v1/projects/3001"),
                Some(&resolved)
            ));
        }
    }

    #[test]
    fn event_without_a_daylite_reference_is_protected() {
        assert!(is_protected_event(None, None));
    }

    #[test]
    fn event_whose_project_lookup_failed_is_not_protected() {
        assert!(!is_protected_event(Some("/v1/projects/3001"), None));
    }
}
