use crate::error::{Error, Result};
use mixpanel_rs::{Config, Mixpanel};
use parking_lot::Mutex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tauri::{AppHandle, Runtime};

use crate::people::MixpanelPeople;
use crate::persistence::{Persistence, PersistenceError, RegisterOptions};

pub struct MixpanelState {
    pub(crate) client: Mixpanel,
    super_properties: Arc<Mutex<HashMap<String, Value>>>,
    persistence: Arc<Persistence>,
    pub people: MixpanelPeople,
}

impl MixpanelState {
    pub fn new<R: Runtime>(
        app_handle: &AppHandle<R>,
        token: &str,
        config: Option<Config>,
    ) -> Result<Self> {
        let client = Mixpanel::init(token, config);
        let persistence = Self::initialize_persistence(app_handle, token)?;

        let initial_props = Self::gather_initial_properties(app_handle, &persistence)?;
        if !initial_props.is_empty() {
            persistence.register(initial_props, None);
        }

        let super_properties = Arc::new(Mutex::new(HashMap::new()));
        let people = MixpanelPeople::new(client.clone(), Arc::clone(&persistence));

        Ok(Self {
            client,
            super_properties,
            persistence,
            people,
        })
    }

    /// Initializes the persistence layer.
    fn initialize_persistence<R: Runtime>(
        app_handle: &AppHandle<R>,
        token: &str,
    ) -> Result<Arc<Persistence>> {
        let persistence_path = app_handle
            .path()
            .app_data_dir()
            .map_err(|_| {
                PersistenceError::PathError("Failed to get app data directory".to_string())
            })?
            .join(format!("mixpanel_{}.json", token));

        Ok(Arc::new(Persistence::new(persistence_path)))
    }

    /// Gathers initial properties (distinct_id, device_id, os, browser, etc.)
    /// to be registered once during initialization.
    fn gather_initial_properties<R: Runtime>(
        app_handle: &AppHandle<R>,
        persistence: &Persistence, // Take persistence as a borrow
    ) -> Result<HashMap<String, Value>> {
        let distinct_id_on_load = persistence.get_distinct_id();
        let device_id_on_load = persistence.get_property("$device_id");

        let mut initial_props: HashMap<String, Value> = HashMap::new();

        if distinct_id_on_load.is_none() || device_id_on_load.is_none() {
            let machine_id = machine_uid::get()
                .map_err(|e| Error::MixpanelError(format!("Failed to get machine ID: {}", e)))?;

            let initial_distinct_id = format!("$device:{}", machine_id);

            if distinct_id_on_load.is_none() {
                persistence.set_distinct_id(Some(initial_distinct_id.clone()));
                initial_props.insert(
                    "distinct_id".to_string(),
                    Value::String(initial_distinct_id),
                );
            }
            if device_id_on_load.is_none() {
                initial_props.insert("$device_id".to_string(), Value::String(machine_id));
            }
        }

        let os_info = tauri_plugin_os::platform();
        let mapped_os = match os_info {
            "macos" => "Mac OS X",
            "windows" => "Windows",
            "linux" => "Linux",
            "ios" => "iOS",
            "android" => "Android",
            _ => os_info,
        };
        initial_props.insert("$os".to_string(), Value::String(mapped_os.to_string()));

        initial_props.insert(
            "$browser".to_string(),
            Value::String("Tauri WebView".to_string()),
        );
        if let Ok(version) = tauri::webview_version() {
            initial_props.insert("$browser_version".to_string(), Value::String(version));
        }

        Ok(initial_props)
    }

    /// Gets the distinct ID currently stored in persistence.
    pub fn get_distinct_id(&self) -> Option<String> {
        self.persistence.get_distinct_id()
    }

    /// Sets the distinct ID in persistence.
    pub fn set_distinct_id(&self, id: Option<String>) {
        self.persistence.set_distinct_id(id);
    }

    /// Registers super properties.
    /// Super properties are added to all subsequent track calls.
    pub async fn register(&self, properties: Value, options: Option<Value>) -> Result<()> {
        let register_options = RegisterOptions::parse_options(options);
        let props_map = self.parse_props(properties)?;

        if register_options.persistent {
            self.persistence.register(props_map, register_options.days);
        } else {
            let mut super_props = self.super_properties.lock();
            super_props.extend(props_map);
        }
        Ok(())
    }

    /// Register a set of super properties only once. This will not
    pub fn register_once(
        &self,
        properties: Value,
        default_value: Option<Value>,
        options: Option<Value>,
    ) -> Result<()> {
        let register_options = RegisterOptions::parse_options(options);
        let props_map = self.parse_props(properties)?;

        if register_options.persistent {
            self.persistence
                .register_once(props_map, default_value, register_options.days);
        } else {
            let mut super_props = self.super_properties.lock();
            for (key, value) in props_map {
                if !super_props.contains_key(&key) {
                    super_props.insert(key.clone(), value);
                } else if let Some(ref dv) = default_value {
                    if super_props.get(&key) == Some(dv) {
                        super_props.insert(key.clone(), value);
                    }
                }
            }
        }
        Ok(())
    }

    /// Unregister a super property.
    /// Removes a super property, preventing it from being sent with future track calls.
    pub fn unregister(&self, property_name: &str, options: Option<Value>) -> Result<()> {
        let register_options = RegisterOptions::parse_options(options);

        if register_options.persistent {
            self.persistence.unregister(property_name);
        } else {
            let mut super_props = self.super_properties.lock();
            super_props.remove(property_name);
        }
        Ok(())
    }

    /// Parses a serde_json::Value into a HashMap<String, Value>.
    fn parse_props(&self, value: Value) -> Result<HashMap<String, Value>> {
        match value {
            Value::Object(obj) => {
                let map = obj.into_iter().collect::<HashMap<_, _>>();
                Ok(map)
            }
            Value::Null => Ok(HashMap::new()),
            _ => Err(Error::MixpanelError(
                "properties must be an object or null".to_string(),
            )),
        }
    }

    /// Gets the value of a single super property.
    /// Checks both persistent and non-persistent properties, prioritizing persistent.
    pub fn get_property(&self, property_name: &str) -> Option<Value> {
        if let Some(value) = self.persistence.get_property(property_name) {
            return Some(value);
        }

        let super_props = self.super_properties.lock();
        super_props.get(property_name).cloned()
    }

    /// Starts a timer for an event.
    /// When the event is tracked using `track()`, the duration since `time_event` was called
    /// will be automatically included as a `$duration` property.
    pub fn time_event(&self, event_name: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        self.persistence
            .set_event_timer(event_name.to_string(), timestamp);
    }

    /// Assigns a user to one or more groups.
    pub async fn set_group(
        &self,
        group_key: &str,
        group_ids: Value,
        options: Option<Value>,
    ) -> Result<()> {
        let group_ids_array = match group_ids {
            Value::Array(arr) => arr,
            Value::String(s) => vec![Value::String(s)],
            Value::Number(n) => vec![Value::Number(n)],
            _ => {
                return Err(Error::MixpanelError(
                    "group_ids must be an array or a single string/number".to_string(),
                ))
            }
        };

        let mut props_map = HashMap::new();
        props_map.insert(group_key.to_string(), Value::Array(group_ids_array.clone()));

        let register_options = RegisterOptions::parse_options(options);
        if register_options.persistent {
            self.persistence.register(props_map, register_options.days);
        } else {
            let mut super_props = self.super_properties.lock();
            super_props.insert(group_key.to_string(), Value::Array(group_ids_array.clone()));
        }

        let mut properties_to_set = HashMap::new();
        properties_to_set.insert(group_key.to_string(), Value::Array(group_ids_array));
        self.people
            .set(Value::Object(properties_to_set.into_iter().collect()), None)
            .await?;

        Ok(())
    }

    /// Adds a single group ID to a group key for the user.
    pub async fn add_group(
        &self,
        group_key: &str,
        group_id: Value,
        options: Option<Value>,
    ) -> Result<()> {
        let group_id_to_add = match group_id {
            Value::String(s) => Value::String(s),
            Value::Number(n) => Value::Number(n),
            _ => {
                return Err(Error::MixpanelError(
                    "group_id must be a single string/number".to_string(),
                ))
            }
        };

        let mut current_groups = self
            .persistence
            .get_property(group_key)
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_else(Vec::new);

        if !current_groups.contains(&group_id_to_add) {
            current_groups.push(group_id_to_add.clone());

            let mut props_map = HashMap::new();
            props_map.insert(group_key.to_string(), Value::Array(current_groups.clone()));

            let register_options = RegisterOptions::parse_options(options);
            if register_options.persistent {
                self.persistence.register(props_map, register_options.days);
            } else {
                let mut super_props = self.super_properties.lock();
                super_props.insert(group_key.to_string(), Value::Array(current_groups));
            }
        }

        let mut properties_to_union = HashMap::new();
        properties_to_union.insert(group_key.to_string(), Value::Array(vec![group_id_to_add]));
        self.people
            .union(
                Value::Object(properties_to_union.into_iter().collect()),
                None,
            )
            .await?;

        Ok(())
    }

    /// Removes a single group ID from a group key for the user.
    pub async fn remove_group(
        &self,
        group_key: &str,
        group_id: Value,
        options: Option<Value>,
    ) -> Result<()> {
        let group_id_to_remove = match group_id {
            Value::String(s) => Value::String(s),
            Value::Number(n) => Value::Number(n),
            _ => {
                return Err(Error::MixpanelError(
                    "group_id must be a single string/number".to_string(),
                ))
            }
        };

        let mut group_list_updated = false;
        if let Some(mut current_groups) = self
            .persistence
            .get_property(group_key)
            .and_then(|v| v.as_array().cloned())
        {
            let initial_len = current_groups.len();
            current_groups.retain(|id| id != &group_id_to_remove);

            if current_groups.len() < initial_len {
                group_list_updated = true;
                let register_options = RegisterOptions::parse_options(options);

                if current_groups.is_empty() {
                    if register_options.persistent {
                        self.persistence.unregister(group_key);
                    } else {
                        let mut super_props = self.super_properties.lock();
                        super_props.remove(group_key);
                    }
                } else {
                    let mut props_map = HashMap::new();
                    props_map.insert(group_key.to_string(), Value::Array(current_groups.clone()));
                    if register_options.persistent {
                        self.persistence.register(props_map, register_options.days);
                    } else {
                        let mut super_props = self.super_properties.lock();
                        super_props.insert(group_key.to_string(), Value::Array(current_groups));
                    }
                }
            }
        }

        if group_list_updated {
            let mut properties_to_remove = HashMap::new();
            properties_to_remove.insert(group_key.to_string(), group_id_to_remove);
            self.people
                .remove(
                    Value::Object(properties_to_remove.into_iter().collect()),
                    None,
                )
                .await?;
        }

        Ok(())
    }

    /// Identifies a user, associating all future events with their profile.
    /// Switches the distinct_id and sends an $identify event.
    pub async fn identify(&self, new_distinct_id: String) -> Result<()> {
        let old_distinct_id_opt = self.get_distinct_id();
        let old_alias_opt = self
            .get_property("$alias")
            .and_then(|v| v.as_str().map(String::from));

        if old_distinct_id_opt.as_ref() != Some(&new_distinct_id) {
            if new_distinct_id.starts_with("$device:") {
                eprintln!("Mixpanel Error: distinct_id cannot have $device: prefix");
                return Ok(());
            }

            if old_alias_opt.as_ref() != Some(&new_distinct_id) {
                if old_alias_opt.is_some() {
                    self.unregister("$alias", None)?;
                }
            }

            let mut user_id_prop = HashMap::new();
            user_id_prop.insert(
                "$user_id".to_string(),
                Value::String(new_distinct_id.clone()),
            );
            self.register(Value::Object(user_id_prop.into_iter().collect()), None)
                .await?;

            if self.persistence.get_property("$device_id").is_none() {
                if let Some(ref old_id) = old_distinct_id_opt {
                    let mut device_props = HashMap::new();
                    device_props.insert("$device_id".to_string(), Value::String(old_id.clone()));
                    device_props
                        .insert("$had_persisted_distinct_id".to_string(), Value::Bool(true));
                    self.register_once(
                        Value::Object(device_props.into_iter().collect()),
                        None,
                        None,
                    )?;
                }
            }

            self.set_distinct_id(Some(new_distinct_id.clone()));
            let mut dist_id_prop = HashMap::new();
            dist_id_prop.insert(
                "distinct_id".to_string(),
                Value::String(new_distinct_id.clone()),
            );
            self.register(Value::Object(dist_id_prop.into_iter().collect()), None)
                .await?;

            if let Some(old_distinct_id) = old_distinct_id_opt {
                let mut identify_props: HashMap<String, Value> = HashMap::new();
                identify_props.insert(
                    "distinct_id".to_string(),
                    Value::String(new_distinct_id.clone()),
                );
                identify_props.insert(
                    "$anon_distinct_id".to_string(),
                    Value::String(old_distinct_id),
                );

                self.client
                    .track("$identify", Some(identify_props))
                    .await
                    .map_err(|e| {
                        Error::MixpanelError(format!("Failed to track $identify event: {}", e))
                    })?;
            }
        }

        Ok(())
    }

    /// Creates an alias, associating a new ID with the current distinct ID.
    pub async fn alias(&self, alias: String, original: Option<String>) -> Result<()> {
        let original_id = match original {
            Some(id) => id,
            None => self.get_distinct_id().ok_or_else(|| {
                Error::MixpanelError("Cannot alias without an existing distinct_id.".to_string())
            })?,
        };

        if alias == original_id {
            println!("Mixpanel: alias matches current distinct_id. Skipping api call.");
            self.identify(alias).await?;
            return Ok(());
        }

        if self
            .persistence
            .get_property("$people_distinct_id")
            .as_ref()
            .and_then(|v| v.as_str())
            == Some(alias.as_str())
        {
            return Err(Error::MixpanelError(
                "Attempting to create alias for existing People user - aborting.".to_string(),
            ));
        }
        let mut alias_prop = HashMap::new();
        alias_prop.insert("$alias".to_string(), Value::String(alias.clone()));
        self.register(Value::Object(alias_prop.into_iter().collect()), None)
            .await?;

        let mut event_props: HashMap<String, Value> = HashMap::new();
        event_props.insert("alias".to_string(), Value::String(alias.clone()));
        event_props.insert(
            "distinct_id".to_string(),
            Value::String(original_id.clone()),
        );

        self.client
            .track("$create_alias", Some(event_props))
            .await
            .map_err(|e| {
                Error::MixpanelError(format!("Failed to track $create_alias event: {}", e))
            })?;

        self.identify(alias).await?;

        Ok(())
    }

    /// Resets the instance, clearing super properties and generating a new distinct ID.
    pub fn reset(&self) -> Result<()> {
        self.persistence.clear_all_data();
        self.super_properties.lock().clear();

        let machine_id = machine_uid::get()
            .map_err(|e| Error::MixpanelError(format!("Failed to get machine ID: {}", e)))?;
        let initial_distinct_id = format!("$device:{}", machine_id);

        let mut props_to_register = HashMap::new();
        props_to_register.insert(
            "distinct_id".to_string(),
            Value::String(initial_distinct_id.clone()),
        );
        props_to_register.insert("$device_id".to_string(), Value::String(machine_id));

        self.register_once(
            Value::Object(props_to_register.into_iter().collect()),
            None,
            None,
        )?;

        Ok(())
    }

    /// Tracks an event with the associated properties.
    /// Merges input properties with superproperties (in-memory and persistent) and adds timing information if available.
    pub async fn track(&self, event_name: String, properties: Option<Value>) -> Result<()> {
        let distinct_id = self.get_distinct_id().ok_or_else(|| {
            Error::MixpanelError("Distinct ID not set. Call identify or alias first.".to_string())
        })?;

        let input_props = self.parse_props(properties.unwrap_or(Value::Null))?;
        let persistent_props = self.persistence.get_properties();
        let memory_props = {
            let memory_props_guard = self.super_properties.lock();
            memory_props_guard.clone()
        };

        let mut final_props = persistent_props;
        final_props.extend(memory_props);
        final_props.extend(input_props);

        if let Some(start_time_ms) = self.persistence.remove_event_timer(&event_name) {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(now_duration) => {
                    let now_ms = now_duration.as_millis();
                    if now_ms >= start_time_ms as u128 {
                        let duration_sec = (now_ms - start_time_ms as u128) as f64 / 1000.0;
                        if let Some(duration_num) = serde_json::Number::from_f64(duration_sec) {
                            final_props
                                .insert("$duration".to_string(), Value::Number(duration_num));
                        } else {
                            eprintln!(
                                "Mixpanel: Could not represent duration {} as f64 for event '{}'",
                                duration_sec, event_name
                            );
                            final_props.insert("$duration".to_string(), Value::Number(0.into()));
                        }
                    } else {
                        eprintln!("Mixpanel: Invalid event timer (start time > current time) detected for event '{}'", event_name);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Mixpanel: Failed to get current system time for duration calculation: {}",
                        e
                    );
                }
            }
        }

        final_props.insert("distinct_id".to_string(), Value::String(distinct_id));
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(now_duration) => {
                final_props.insert(
                    "time".to_string(),
                    Value::Number(now_duration.as_secs().into()),
                );
            }
            Err(e) => {
                eprintln!(
                    "Mixpanel: Failed to get current system time for event timestamp: {}",
                    e
                );
                final_props.insert("time".to_string(), Value::Number(0.into()));
            }
        }

        self.client
            .track(&event_name, Some(final_props))
            .await
            .map_err(|e| {
                Error::MixpanelError(format!("Failed to track event '{}': {}", event_name, e))
            })?;

        Ok(())
    }
}
