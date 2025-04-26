# Tauri Plugin Mixpanel

[![crates.io](https://img.shields.io/crates/v/tauri-plugin-mixpanel.svg)](https://crates.io/crates/tauri-plugin-mixpanel)
[![documentation](https://docs.rs/tauri-plugin-mixpanel/badge.svg)](https://docs.rs/tauri-plugin-mixpanel)

This plugin provides a Rust wrapper and TypeScript bindings for using [Mixpanel](https://mixpanel.com/) analytics within your Tauri application. It leverages the [`mixpanel-rs`](https://github.com/ahonn/mixpanel-rs) crate for the core Mixpanel interactions.

## Features

*   Track events with custom properties.
*   Identify users with unique IDs.
*   Manage user profiles (People operations: set, set\_once, increment, append, etc.).
*   Manage user groups.
*   Time events.
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

```typescript
import { mixpanel, MixpanelError } from 'tauri-plugin-mixpanel-api';

async function setupMixpanel() {
  try {
    // Identify the user
    await mixpanel.identify('user-123');

    // Set some user profile properties (Mixpanel People)
    await mixpanel.people.set({
      $email: 'user@example.com',
      plan: 'Premium',
    });

    // Register super properties (sent with every event)
    await mixpanel.register({
      appName: 'My Tauri App',
      appVersion: '1.0.0',
    });

    // Track an event
    await mixpanel.track('App Launched', { source: 'desktop' });

    // Time an event
    mixpanel.time_event('Data Processing');
    // ... perform data processing ...
    await mixpanel.track('Data Processing'); // This will include the duration

    // Set user groups
    await mixpanel.set_group('company', 'company-abc');
    await mixpanel.add_group('project', 'project-xyz');

  } catch (error) {
    if (error instanceof MixpanelError) {
      console.error('Mixpanel Error:', error.message);
    } else {
      console.error('Unknown Error:', error);
    }
  }
}

setupMixpanel();

// --- Example: Tracking a button click ---
const myButton = document.getElementById('my-button');
myButton?.addEventListener('click', async () => {
  try {
    await mixpanel.track('Button Clicked', { buttonId: 'my-button' });
    console.log('Button click tracked');
  } catch (error) {
     if (error instanceof MixpanelError) {
      console.error('Mixpanel Error:', error.message);
    } else {
      console.error('Unknown Error:', error);
    }
  }
});

// --- Example: Get Distinct ID ---
async function logDistinctId() {
    const distinctId = await mixpanel.get_distinct_id();
    console.log('Current Mixpanel Distinct ID:', distinctId);
}

logDistinctId();

// --- Resetting (e.g., on logout) ---
async function logoutUser() {
    // Perform logout actions...

    // Reset Mixpanel (clears distinct ID and super properties)
    await mixpanel.reset();
    console.log('Mixpanel reset.');
}

```

## API

The TypeScript API mirrors the standard Mixpanel JavaScript library methods closely. Refer to the `guest-js/index.ts` file and the official [Mixpanel JavaScript Library Reference](https://developer.mixpanel.com/docs/javascript) for detailed method descriptions.

Key methods include:

*   `mixpanel.identify(distinctId)`
*   `mixpanel.track(eventName, properties?)`
*   `mixpanel.register(properties)`
*   `mixpanel.register_once(properties, defaultValue?)`
*   `mixpanel.unregister(propertyName)`
*   `mixpanel.get_distinct_id()`
*   `mixpanel.get_property(propertyName)`
*   `mixpanel.reset()`
*   `mixpanel.time_event(eventName)`
*   `mixpanel.set_group(groupKey, groupId)`
*   `mixpanel.add_group(groupKey, groupId)`
*   `mixpanel.remove_group(groupKey, groupId)`
*   `mixpanel.people.set(properties)`
*   `mixpanel.people.set_once(properties)`
*   `mixpanel.people.unset(properties)`
*   `mixpanel.people.increment(properties, amount?)`
*   `mixpanel.people.append(listName, value)`
*   `mixpanel.people.remove(listName, value)`
*   `mixpanel.people.union(listName, values)`
*   `mixpanel.people.delete_user()`

## Permissions

This plugin currently does not require any specific capabilities to be enabled in your `tauri.conf.json` allowlist, as it interacts with the network via the Rust core, not directly from the webview.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.

## License

This project is licensed under the MIT License.
