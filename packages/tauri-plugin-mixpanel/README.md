# Tauri Plugin Mixpanel

A Tauri plugin for integrating Mixpanel analytics into your Tauri application.

## Installation

### Rust

Add the following to your `Cargo.toml`:

```toml
[dependencies]
tauri-plugin-mixpanel = "0.1.0"
```

### JavaScript/TypeScript

```bash
npm install tauri-plugin-mixpanel-api
# or
yarn add tauri-plugin-mixpanel-api
# or
pnpm add tauri-plugin-mixpanel-api
```

## Usage

### Rust

```rust
use tauri_plugin_mixpanel::Builder;

fn main() {
    tauri::Builder::default()
        .plugin(
            Builder::new("YOUR_MIXPANEL_TOKEN")
                .api_host("https://api.mixpanel.com") // Optional
                .disable_sending(false) // Optional
                .build()
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### JavaScript/TypeScript

```typescript
import { track, identify, alias, optIn, optOut, isOptedOut } from 'tauri-plugin-mixpanel-api';

// Track an event
await track('button_clicked', { buttonId: 'submit', page: 'homepage' });

// Identify a user
await identify('user123', { name: 'John Doe', email: 'john@example.com' });

// Create an alias
await alias('user123', 'john_doe');

// Opt in/out
await optIn();
await optOut();
const isUserOptedOut = await isOptedOut();
```

## Features

- Event tracking
- User identification
- User aliasing
- Opt-in/opt-out for GDPR compliance
- Customizable API host

## License

MIT
