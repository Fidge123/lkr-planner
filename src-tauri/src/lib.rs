use tauri_plugin_updater::UpdaterExt;

mod integrations;
pub mod secret_manager;

fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        integrations::health::check_health,
        integrations::local_store::load_local_store,
        integrations::local_store::save_local_store,
        integrations::calendar::commands::load_week_events,
        integrations::holidays::get_holidays_for_week,
        integrations::daylite::auth::daylite_connect_refresh_token,
        integrations::daylite::projects::daylite_list_projects,
        integrations::daylite::projects::daylite_search_projects,
        integrations::daylite::projects::daylite_query_overdue_projects,
        integrations::daylite::contacts::commands::daylite_list_contacts,
        integrations::daylite::contacts::commands::daylite_list_cached_contacts,
        integrations::daylite::contacts::commands::daylite_update_contact_ical_urls,
        integrations::planradar::auth::planradar_connect,
        integrations::planradar::projects::planradar_get_project_status,
        integrations::planradar::projects::planradar_list_projects,
        integrations::planradar::projects::planradar_create_project,
        integrations::planradar::projects::planradar_copy_project,
        integrations::planradar::projects::planradar_reactivate_project,
        integrations::calendar::commands::create_assignment,
        integrations::calendar::commands::update_assignment,
        integrations::calendar::commands::delete_assignment,
        integrations::zep::commands::zep_save_credentials,
        integrations::zep::commands::zep_load_credentials,
        integrations::zep::commands::zep_test_credentials,
        integrations::zep::commands::zep_discover_calendars,
        integrations::zep::commands::zep_save_and_test_calendar,
    ])
}

fn export_bindings(specta_builder: &tauri_specta::Builder<tauri::Wry>) {
    specta_builder
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck"),
            "../src/generated/tauri.ts",
        )
        .expect("failed to export tauri specta bindings");
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder = specta_builder();

    #[cfg(debug_assertions)]
    export_bindings(&specta_builder);

    tauri::Builder::default()
        .setup(|app| {
            if let Err(error) = secret_manager::init() {
                eprintln!("Failed to initialize credential store: {error}");
            }

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(message) = format_update_error(update(handle).await) {
                    eprintln!("{message}");
                }
            });
            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        update
            .download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                    println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

    Ok(())
}

fn format_update_error<E: std::fmt::Display>(result: Result<(), E>) -> Option<String> {
    result
        .err()
        .map(|error| format!("Update check failed in background task: {error}"))
}

#[cfg(test)]
mod bindings {
    use super::{export_bindings, specta_builder};

    #[test]
    fn regenerate_generated_bindings() {
        export_bindings(&specta_builder());
    }
}
