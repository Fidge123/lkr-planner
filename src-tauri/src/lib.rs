use tauri_plugin_updater::UpdaterExt;

mod integrations;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let specta_builder =
        tauri_specta::Builder::<tauri::Wry>::new().commands(tauri_specta::collect_commands![
            integrations::health::check_health,
            integrations::local_store::load_local_store,
            integrations::local_store::save_local_store,
            integrations::daylite::auth::daylite_connect_refresh_token,
            integrations::daylite::projects::daylite_list_projects,
            integrations::daylite::projects::daylite_search_projects,
            integrations::daylite::contacts::daylite_list_contacts,
            integrations::daylite::contacts::daylite_list_cached_contacts,
            integrations::daylite::contacts::daylite_update_contact_ical_urls
        ]);

    specta_builder
        .export(
            specta_typescript::Typescript::default().header("// @ts-nocheck"),
            "../src/generated/tauri.ts",
        )
        .expect("failed to export tauri specta bindings");

    tauri::Builder::default()
        .setup(|app| {
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
