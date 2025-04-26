#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .plugin(
            tauri_plugin_mixpanel::Builder::new(
                "24b2353d5f029dd13c29a3a1389bd6b5",
                Some(tauri_plugin_mixpanel::Config {
                    debug: true,
                    geolocate: true,
                    ..Default::default()
                }),
            )
            .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
