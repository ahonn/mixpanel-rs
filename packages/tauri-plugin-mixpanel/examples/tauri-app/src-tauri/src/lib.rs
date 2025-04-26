use tauri_plugin_mixpanel::{MixpanelExt, Config};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_mixpanel::Builder::new(
                "24b2353d5f029dd13c29a3a1389bd6b5",
                Some(Config {
                    debug: true,
                    geolocate: true,
                    ..Default::default()
                }),
            )
            .build(),
        )
        .setup(|app| {
            let handle = app.handle().clone();
            tokio::spawn(async move {
                let mixpanel_state = handle.mixpanel();

                let distinct_id = mixpanel_state.get_distinct_id();
                println!("[Rust Setup] Initial Distinct ID: {:?}", distinct_id);

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                if let Err(e) = mixpanel_state.register(json!({ "Setup Time": now }), None).await {
                    eprintln!("[Rust Setup] Failed to register property: {}", e);
                }

                if let Err(e) = mixpanel_state.track("Rust Setup".to_string(), Some(json!({ "status": "success" }))).await {
                    eprintln!("[Rust Setup] Failed to track event: {}", e);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
