use crate::integrations::telemetry::events::{Integration, Operation};
use crate::integrations::telemetry::observe::observe;
use tauri_plugin_opener::OpenerExt;

#[tauri::command]
#[specta::specta]
pub async fn daylite_open_project(
    app: tauri::AppHandle,
    project_ref: String,
) -> Result<(), String> {
    let handle = app.clone();
    observe(
        &handle,
        Operation::DayliteOpenProject,
        Integration::Daylite,
        async { daylite_open_project_inner(app, project_ref) },
    )
    .await
}

fn daylite_open_project_inner(app: tauri::AppHandle, project_ref: String) -> Result<(), String> {
    let url = project_deep_link_url(&project_ref)?;
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|_| OPEN_FAILED_MESSAGE.to_string())
}

const OPEN_FAILED_MESSAGE: &str = "Das Projekt konnte in Daylite nicht geöffnet werden.";

/// Not a query string and not `daylite4:`: the parameters sit in the authority position,
/// and a URL type would normalize both away.
fn project_deep_link_url(project_ref: &str) -> Result<String, String> {
    let id = project_ref
        .trim()
        .strip_prefix("/v1/projects/")
        .filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| OPEN_FAILED_MESSAGE.to_string())?;

    Ok(format!(
        "daylite://Command=ShowObject&Entity=Project&ID={id}"
    ))
}

#[cfg(test)]
mod tests {
    use super::project_deep_link_url;

    #[test]
    fn builds_the_show_object_url_from_a_project_reference() {
        assert_eq!(
            project_deep_link_url("/v1/projects/2035"),
            Ok("daylite://Command=ShowObject&Entity=Project&ID=2035".to_string())
        );
    }

    #[test]
    fn rejects_a_reference_that_does_not_name_a_project() {
        assert_eq!(
            project_deep_link_url("/v1/contacts/2035"),
            Err("Das Projekt konnte in Daylite nicht geöffnet werden.".to_string())
        );
    }

    #[test]
    fn rejects_a_project_reference_without_a_numeric_id() {
        assert!(project_deep_link_url("/v1/projects/abc").is_err());
        assert!(project_deep_link_url("/v1/projects/").is_err());
        assert!(project_deep_link_url("").is_err());
    }
}
