# Tauri Plugin Mixpanel

[![crates.io](https://img.shields.io/crates/v/tauri-plugin-mixpanel.svg)](https://crates.io/crates/tauri-plugin-mixpanel)
[![documentation](https://docs.rs/tauri-plugin-mixpanel/badge.svg)](https://docs.rs/tauri-plugin-mixpanel)

This plugin provides a Rust wrapper and TypeScript bindings for using [Mixpanel](https://mixpanel.com/) analytics within your Tauri application. It leverages the [`mixpanel-rs`](https://github.com/ahonn/mixpanel-rs) crate for the core Mixpanel interactions.

## Features

*   Track events with custom properties.
*   Identify users with unique IDs.
*   Manage user profiles.
*   Persistent super properties.
*   Offline persistence and automatic batching of events.

## Install

### Rust

Add the plugin to your `Cargo.toml` dependencies:

```toml
[dependencies]
tauri-plugin-mixpanel = { git = "https://github.com/ahonn/mixpanel-rs", branch = "main" }
# Or from crates.io:
# tauri-plugin-mixpanel = "<version>"
```

Register the plugin in your `main.rs`:

```rust
use tauri_plugin_mixpanel::{self, Config};

fn main() {
    let mixpanel_token = "YOUR_MIXPANEL_TOKEN"; // Replace with your actual token
    let mixpanel_config = Some(Config {
        // Optional: Configure batch size, flush interval, etc.
        // See mixpanel-rs docs for details: https://docs.rs/mixpanel-rs
        ..Default::default()
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_mixpanel::Builder::new(mixpanel_token, mixpanel_config).build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### JavaScript/TypeScript

Install the frontend bindings using your preferred package manager:

```bash
npm install @tauri-apps/api tauri-plugin-mixpanel-api
# or
yarn add @tauri-apps/api tauri-plugin-mixpanel-api
# or
pnpm add @tauri-apps/api tauri-plugin-mixpanel-api
```

## Usage (TypeScript)

Import the bindings and use them in your frontend code :

```typescript
import mixpanel from 'tauri-plugin-mixpanel-api';

async function setupAnalytics() {
  await mixpanel.identify("user_12345");
  await mixpanel.people.set({ "$name": "Alice", "plan": "Premium" });
  await mixpanel.register({ "App Version": "1.2.0" });
  await mixpanel.track("App Started", { "source": "Frontend" });
  console.log("Mixpanel initialized and event tracked.");
}

setupAnalytics();
```

## Usage (Rust)

You can interact with the Mixpanel instance directly from your Rust backend code using Tauri's state management and the `MixpanelExt` trait.

First, ensure the plugin is registered as shown in the installation steps.

### Accessing Mixpanel State

You can get the managed `Mixpanel` state in several ways:

1.  **In Tauri Commands:** Use `tauri::State<'_, MixpanelState>`.
2.  **Usage Outside Commands** Use `app.state::<MixpanelState>()` or the `app.handle().mixpanel()` extension method.

### Example: Tracking from a Command

```rust
use tauri::State;
use tauri_plugin_mixpanel::{MixpanelState, Error as MixpanelError};
use serde_json::json;

#[tauri::command]
async fn track_backend_action(
    event_name: String,
    mixpanel: State<'_, MixpanelState>
) -> Result<(), String> {
    let props = json!({
        "source": "Rust Command",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    mixpanel.track(&event_name, Some(props)).await
        .map_err(|e: MixpanelError| e.to_string())?;

    println!("[Rust Command] Tracked event: {}", event_name);
    Ok(())
}
```

### Example: Usage Outside Commands

To use the Mixpanel client from Rust code outside of a Tauri command (like in the `setup` hook, event listeners, or background tasks), you need access to the Tauri `AppHandle`. You can then use the `MixpanelExt` trait to get the client.

This often involves cloning the `AppHandle` and moving it into an async task.

```rust
use tauri::{AppHandle, Manager};
use tauri_plugin_mixpanel::{MixpanelExt, Config, Error as MixpanelError};
use serde_json::json;

fn perform_background_mixpanel_actions(handle: AppHandle) {
    tokio::spawn(async move {
        let mixpanel = handle.mixpanel();

        let distinct_id = mixpanel.get_distinct_id().await;
        println!("[Background Task] Current Distinct ID: {:?}", distinct_id);

        let register_props = json!({ 
            "last_background_run": chrono::Utc::now().to_rfc3339(),
            "background_task_id": rand::random::<u32>() // Example: requires `rand` crate
        });
        if let Err(e) = mixpanel.register(register_props, None).await {
            eprintln!("[Background Task] Failed to register property: {}", e);
        }

        if let Err(e) = mixpanel.track("Background Task Started", None).await {
            eprintln!("[Background Task] Failed to track event: {}", e);
        }
    });
}
```

## Permissions

This plugin currently does not require any specific capabilities to be enabled in your `tauri.conf.json` allowlist, as it interacts with the network via the Rust core, not directly from the webview.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License.
