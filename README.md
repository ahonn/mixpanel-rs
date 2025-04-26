# mixpanel-rs

[![crates.io](https://img.shields.io/crates/v/mixpanel-rs.svg)](https://crates.io/crates/mixpanel-rs)
[![documentation](https://docs.rs/mixpanel-rs/badge.svg)](https://docs.rs/mixpanel-rs)

An asynchronous Rust client for interacting with the [Mixpanel](https://mixpanel.com/) API, inspired by the official Node.js library.

## Features

*   Track events (`track`, `track_batch`)
*   Manage user profiles (People API: `set`, `set_once`, `increment`, `append`, `union`, `remove`, `unset`, `delete_user`)
*   Manage group profiles (Groups API: `set`, `set_once`, `remove`, `union`, `delete_group`)
*   Configurable API endpoint and behavior (debug, test mode)

## Installation

Add `mixpanel-rs` to your `Cargo.toml` dependencies:

```toml
[dependencies]
mixpanel-rs = "<latest-version>" # Replace with the actual latest version
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"
# Optional: for loading env vars from .env
dotenv = "0.15"
```

## Usage

### Initialization

First, get your Project Token and optionally your API Secret from your Mixpanel project settings.

```rust
use mixpanel_rs::{Mixpanel, Config};
use std::env;
use dotenv::dotenv;

// Load .env file if present
dotenv().ok();

let project_token = env::var("MIXPANEL_PROJECT_TOKEN")
    .expect("MIXPANEL_PROJECT_TOKEN must be set");
// API Secret is needed for import endpoints
let api_secret = env::var("MIXPANEL_API_SECRET").ok();

// Optional configuration
let config = Config {
    secret: api_secret, // Required for import calls
    debug: true,        // Log requests
    // host: "api-eu.mixpanel.com", // Use EU residency server if needed
    ..Default::default()
};

let mp = Mixpanel::init(&project_token, Some(config));
```

### Tracking Events

```rust
use mixpanel_rs::{Mixpanel, Config};
use serde_json::json;
use std::collections::HashMap;

# let project_token = "token";
# let mp = Mixpanel::init(project_token, None);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple event
    mp.track("App Launched", None).await?;

    // Event with properties
    let mut properties = HashMap::new();
    properties.insert("Plan".to_string(), json!("Premium"));
    properties.insert("User Type".to_string(), json!("Paid"));
    mp.track("Signed Up", Some(properties)).await?;

    Ok(())
}
```

### People API (User Profiles)

```rust
use mixpanel_rs::{Mixpanel, Config, Modifiers};
use serde_json::json;
use std::collections::HashMap;

# let project_token = "token";
# let mp = Mixpanel::init(project_token, None);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let distinct_id = "user_distinct_id_123";
    let modifiers = Some(Modifiers::default()); // Optional: Modifiers like $ip, $time, etc.

    // Set user properties
    let mut set_props = HashMap::new();
    set_props.insert("$name".to_string(), json!("Alice Example"));
    set_props.insert("$email".to_string(), json!("alice@example.com"));
    set_props.insert("Plan".to_string(), json!("Free"));
    mp.people.set(distinct_id, set_props, modifiers.clone()).await?;

    // Set properties only once
    let mut once_props = HashMap::new();
    once_props.insert("First Login Date".to_string(), json!(Mixpanel::now()));
    mp.people.set_once(distinct_id, once_props, modifiers.clone()).await?;

    // Increment numeric properties
    let mut inc_props = HashMap::new();
    inc_props.insert("Logins".to_string(), 1);
    inc_props.insert("Credits Used".to_string(), 15);
    mp.people.increment(distinct_id, inc_props, modifiers.clone()).await?;

    // Append to a list property
    let mut append_props = HashMap::new();
    append_props.insert("Items Purchased".to_string(), json!("T-Shirt"));
    mp.people.append(distinct_id, append_props, modifiers.clone()).await?;

    // See people.rs and groups.rs examples for more operations like:
    // unset, remove, union, delete_user

    Ok(())
}
```

## Error Handling

The API methods return `mixpanel_rs::Result<T>`, which is an alias for `std::result::Result<T, mixpanel_rs::Error>`. Handle potential errors like network issues or API errors.

```rust
# use mixpanel_rs::{Mixpanel, Config};
# let project_token = "token";
# let mp = Mixpanel::init(project_token, None);
#[tokio::main]
async fn main() {
    match mp.track("Test Event", None).await {
        Ok(_) => println!("Event tracked successfully!"),
        Err(e) => eprintln!("Failed to track event: {}", e),
    }
}
```

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## License

This project is licensed under the MIT License. 