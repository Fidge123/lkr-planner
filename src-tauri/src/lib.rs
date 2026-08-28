use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::Emitter;
use tauri_plugin_updater::UpdaterExt;

mod integrations;
pub mod secret_manager;

const RELOAD_DATA_MENU_ID: &str = "reload-data";
/// Matched by the frontend listener in `use-reload-data-menu.ts`.
const RELOAD_DATA_EVENT: &str = "reload-data";

fn specta_builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
        integrations::local_store::load_local_store,
        integrations::local_store::save_local_store,
        integrations::calendar::commands::load_week_events,
        integrations::holidays::get_holidays_for_week,
        integrations::daylite::auth::daylite_connect_refresh_token,
        integrations::daylite::projects::daylite_search_projects,
        integrations::daylite::projects::daylite_query_overdue_projects,
        integrations::daylite::categories::daylite_project_category_colors,
        integrations::daylite::deep_link::daylite_open_project,
        integrations::daylite::contacts::commands::daylite_list_contacts,
        integrations::daylite::contacts::commands::daylite_list_cached_contacts,
        integrations::calendar::commands::create_assignment,
        integrations::calendar::commands::update_assignment,
        integrations::calendar::commands::move_assignment,
        integrations::calendar::commands::reorder_assignment,
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

            install_reload_data_menu(app.handle())?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(message) = format_update_error(update(handle).await) {
                    eprintln!("{message}");
                }
            });
            Ok(())
        })
        .on_menu_event(|app, event| {
            if event.id() == RELOAD_DATA_MENU_ID {
                if let Err(error) = app.emit(RELOAD_DATA_EVENT, ()) {
                    eprintln!("Failed to emit reload event: {error}");
                }
            }
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn install_reload_data_menu(app: &tauri::AppHandle) -> tauri::Result<()> {
    let menu = Menu::default(app)?;
    let reload = MenuItem::with_id(
        app,
        RELOAD_DATA_MENU_ID,
        "Daten neu laden",
        true,
        None::<&str>,
    )?;

    match file_submenu(&menu)? {
        Some(file) => file.prepend(&reload)?,
        // Only reachable if a platform's default menu ever ships without a File submenu.
        None => menu.insert(&Submenu::with_items(app, "File", true, &[&reload])?, 1)?,
    }

    app.set_menu(menu)?;
    Ok(())
}

fn file_submenu(menu: &Menu<tauri::Wry>) -> tauri::Result<Option<Submenu<tauri::Wry>>> {
    for item in menu.items()? {
        let Some(submenu) = item.as_submenu() else {
            continue;
        };
        if submenu.text()? == "File" {
            return Ok(Some(submenu.clone()));
        }
    }

    Ok(None)
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
