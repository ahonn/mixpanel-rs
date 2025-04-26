use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Path error: {0}")]
    PathError(String),
    #[error("Lock error: {0}")]
    LockError(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegisterOptions {
    #[serde(default = "default_persistent")]
    pub persistent: bool,
    pub days: Option<u64>,
}

fn default_persistent() -> bool {
    true
}

impl Default for RegisterOptions {
    fn default() -> Self {
        RegisterOptions {
            persistent: true,
            days: None,
        }
    }
}

impl RegisterOptions {
    pub fn parse_options(options: Option<Value>) -> RegisterOptions {
        match options {
            Some(Value::Object(mut map)) => {
                let persistent = map
                    .remove("persistent")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let days = map.remove("days").and_then(|v| v.as_u64());

                RegisterOptions { persistent, days }
            }
            _ => RegisterOptions::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub(crate) struct PersistentData {
    distinct_id: Option<String>,
    alias: Option<String>,
    event_timers: HashMap<String, u64>,
    properties: HashMap<String, Value>,
    store_expires_at: Option<u64>,
}

pub(crate) struct Persistence {
    pub(crate) path: PathBuf,
    pub(crate) data: Arc<RwLock<PersistentData>>,
}

fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

impl Persistence {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let path_buf = path.as_ref().to_path_buf();
        let initial_data = match Self::load_sync(&path_buf) {
            Ok(data) => data,
            Err(e) => {
                eprintln!(
                    "[Mixpanel Persistence] Failed to load initial data from {}: {}. Starting fresh.",
                    path_buf.display(),
                    e
                );
                PersistentData::default()
            }
        };

        Persistence {
            path: path_buf,
            data: Arc::new(RwLock::new(initial_data)),
        }
    }

    fn load_sync(path: &PathBuf) -> Result<PersistentData, PersistenceError> {
        if !path.exists() {
            return Ok(PersistentData::default());
        }
        let contents = std::fs::read_to_string(path)?;
        let data: PersistentData = serde_json::from_str(&contents)?;

        let now = current_time_millis();
        if let Some(expires_at) = data.store_expires_at {
            if now >= expires_at {
                return Ok(PersistentData::default());
            }
        }
        Ok(data)
    }

    async fn write_data_async(
        &self,
        data_to_write: PersistentData,
    ) -> Result<(), PersistenceError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = File::create(&self.path).await?;
        let contents = serde_json::to_string_pretty(&data_to_write)?;
        file.write_all(contents.as_bytes()).await?;
        Ok(())
    }

    fn trigger_save(&self) {
        match self.data.read() {
            Ok(data_guard) => {
                let data_clone = data_guard.clone();
                let path_clone = self.path.clone();
                tauri::async_runtime::spawn(async move {
                    let persistence = Persistence {
                        path: path_clone,
                        data: Arc::new(RwLock::new(PersistentData::default())),
                    };
                    if let Err(e) = persistence.write_data_async(data_clone).await {
                        eprintln!("[Mixpanel Persistence] Failed to save data: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!(
                    "[Mixpanel Persistence] Failed to acquire read lock for saving: {}",
                    e
                );
            }
        }
    }

    pub fn register(&self, props: HashMap<String, Value>, days: Option<u64>) {
        match self.data.write() {
            Ok(mut data_guard) => {
                data_guard.properties.extend(props);

                if let Some(d) = days {
                    if d > 0 {
                        let expiration_duration = Duration::from_secs(d * 24 * 60 * 60);
                        let expires_at =
                            current_time_millis() + expiration_duration.as_millis() as u64;
                        if data_guard.store_expires_at.map_or(true, |current_exp| {
                            expires_at > current_exp || current_time_millis() >= current_exp
                        }) {
                            data_guard.store_expires_at = Some(expires_at);
                        }
                    } else {
                        data_guard.store_expires_at = None;
                    }
                }
                drop(data_guard);
                self.trigger_save();
            }
            Err(e) => eprintln!("[Mixpanel Persistence] Lock error during register: {}", e),
        }
    }

    pub fn register_once(
        &self,
        props: HashMap<String, Value>,
        default_value: Option<Value>,
        days: Option<u64>,
    ) {
        match self.data.write() {
            Ok(mut data_guard) => {
                let mut changed = false;
                for (key, value) in props {
                    match data_guard.properties.get(&key) {
                        Some(existing_val) => {
                            if let Some(ref default) = default_value {
                                if existing_val == default {
                                    data_guard.properties.insert(key.clone(), value);
                                    changed = true;
                                }
                            }
                        }
                        None => {
                            data_guard.properties.insert(key.clone(), value);
                            changed = true;
                        }
                    }
                }

                if changed {
                    if let Some(d) = days {
                        if d > 0 {
                            let expiration_duration = Duration::from_secs(d * 24 * 60 * 60);
                            let expires_at =
                                current_time_millis() + expiration_duration.as_millis() as u64;
                            if data_guard.store_expires_at.map_or(true, |current_exp| {
                                expires_at > current_exp || current_time_millis() >= current_exp
                            }) {
                                data_guard.store_expires_at = Some(expires_at);
                            }
                        } else {
                            data_guard.store_expires_at = None;
                        }
                    }
                }

                drop(data_guard);
                if changed {
                    self.trigger_save();
                }
            }
            Err(e) => eprintln!(
                "[Mixpanel Persistence] Lock error during register_once: {}",
                e
            ),
        }
    }

    pub fn unregister(&self, property_name: &str) {
        match self.data.write() {
            Ok(mut data_guard) => {
                let changed = data_guard.properties.remove(property_name).is_some();
                drop(data_guard);
                if changed {
                    self.trigger_save();
                }
            }
            Err(e) => eprintln!("[Mixpanel Persistence] Lock error during unregister: {}", e),
        }
    }

    pub fn get_properties(&self) -> HashMap<String, Value> {
        match self.data.read() {
            Ok(data_guard) => {
                let now = current_time_millis();
                if let Some(expires_at) = data_guard.store_expires_at {
                    if now >= expires_at {
                        return HashMap::new();
                    }
                }
                data_guard.properties.clone()
            }
            Err(e) => {
                eprintln!(
                    "[Mixpanel Persistence] Lock error during get_properties: {}",
                    e
                );
                HashMap::new()
            }
        }
    }

    /// Retrieves a single property value by its key.
    /// Returns None if the property doesn't exist or the store is expired.
    pub fn get_property(&self, key: &str) -> Option<Value> {
        match self.data.read() {
            Ok(data_guard) => {
                let now = current_time_millis();
                if let Some(expires_at) = data_guard.store_expires_at {
                    if now >= expires_at {
                        return None;
                    }
                }
                data_guard.properties.get(key).cloned()
            }
            Err(e) => {
                eprintln!(
                    "[Mixpanel Persistence] Lock error during get_property for key '{}': {}",
                    key, e
                );
                None
            }
        }
    }

    pub fn get_distinct_id(&self) -> Option<String> {
        self.data.read().ok().and_then(|d| d.distinct_id.clone())
    }

    pub fn set_distinct_id(&self, id: Option<String>) {
        match self.data.write() {
            Ok(mut data_guard) => {
                data_guard.distinct_id = id;
                drop(data_guard);
                self.trigger_save();
            }
            Err(e) => eprintln!(
                "[Mixpanel Persistence] Lock error during set_distinct_id: {}",
                e
            ),
        }
    }

    pub fn set_event_timer(&self, event: String, timestamp: u64) {
        match self.data.write() {
            Ok(mut data_guard) => {
                data_guard.event_timers.insert(event, timestamp);
                drop(data_guard);
                self.trigger_save();
            }
            Err(e) => eprintln!(
                "[Mixpanel Persistence] Lock error during set_event_timer: {}",
                e
            ),
        }
    }

    pub fn remove_event_timer(&self, event: &str) -> Option<u64> {
        match self.data.write() {
            Ok(mut data_guard) => {
                let removed_timer = data_guard.event_timers.remove(event);
                drop(data_guard);
                if removed_timer.is_some() {
                    self.trigger_save();
                }
                removed_timer
            }
            Err(e) => {
                eprintln!(
                    "[Mixpanel Persistence] Lock error during remove_event_timer: {}",
                    e
                );
                None
            }
        }
    }

    pub fn clear_all_data(&self) {
        match self.data.write() {
            Ok(mut data_guard) => {
                *data_guard = PersistentData::default();
                drop(data_guard);
                self.trigger_save();
                let path_clone = self.path.clone();
                tokio::spawn(async move {
                    match fs::remove_file(path_clone).await {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                        Err(e) => eprintln!(
                            "[Mixpanel Persistence] Failed to delete persistence file on clear: {}",
                            e
                        ),
                    }
                });
            }
            Err(e) => eprintln!(
                "[Mixpanel Persistence] Lock error during clear_all_data: {}",
                e
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs as std_fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn setup_test_persistence(test_name: &str) -> (Persistence, PathBuf) {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join(format!("mixpanel_test_{}.json", test_name));
        let persistence = Persistence::new(&file_path);
        std::mem::forget(dir);
        (persistence, file_path)
    }

    fn cleanup_test_file(path: &PathBuf) {
        let _ = std_fs::remove_file(path);
        if let Some(parent) = path.parent() {
            let _ = std_fs::remove_dir_all(parent); // Clean up the temp dir
        }
    }

    async fn wait_for_save() {
        tokio::time::sleep(Duration::from_millis(50)).await; // Adjust timing if needed
    }

    async fn read_test_file(path: &PathBuf) -> Result<PersistentData, PersistenceError> {
        if !path.exists() {
            return Ok(PersistentData::default());
        }
        let contents = fs::read_to_string(path).await?;
        serde_json::from_str(&contents).map_err(PersistenceError::from)
    }

    #[tokio::test]
    async fn test_new_persistence_creates_default_when_no_file() {
        let (persistence, file_path) = setup_test_persistence("new_default");
        assert!(persistence.get_distinct_id().is_none());
        assert!(persistence.get_properties().is_empty());
        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_register_and_get_properties() {
        let (persistence, file_path) = setup_test_persistence("register_get");
        let mut props = HashMap::new();
        props.insert("key1".to_string(), json!("value1"));
        props.insert("key2".to_string(), json!(123));

        persistence.register(props.clone(), None);
        wait_for_save().await; // Allow time for async save

        let retrieved_props = persistence.get_properties();
        assert_eq!(retrieved_props.len(), 2);
        assert_eq!(retrieved_props.get("key1"), Some(&json!("value1")));
        assert_eq!(retrieved_props.get("key2"), Some(&json!(123)));

        let file_data = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data.properties.len(), 2);
        assert_eq!(file_data.properties.get("key1"), Some(&json!("value1")));

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_register_once() {
        let (persistence, file_path) = setup_test_persistence("register_once");
        let mut initial_props = HashMap::new();
        initial_props.insert("key1".to_string(), json!("initial"));
        initial_props.insert("key2".to_string(), json!("initial_to_overwrite"));
        persistence.register(initial_props, None);
        wait_for_save().await;

        let mut new_props = HashMap::new();
        new_props.insert("key1".to_string(), json!("new_value")); // Should not overwrite
        new_props.insert("key2".to_string(), json!("new_value_overwrite")); // Should overwrite if default matches
        new_props.insert("key3".to_string(), json!("new_key")); // Should add

        persistence.register_once(new_props, Some(json!("initial_to_overwrite")), None);
        wait_for_save().await;

        let props = persistence.get_properties();
        assert_eq!(props.get("key1"), Some(&json!("initial")));
        assert_eq!(props.get("key2"), Some(&json!("new_value_overwrite")));
        assert_eq!(props.get("key3"), Some(&json!("new_key")));

        let file_data = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data.properties.get("key1"), Some(&json!("initial")));
        assert_eq!(
            file_data.properties.get("key2"),
            Some(&json!("new_value_overwrite"))
        );
        assert_eq!(file_data.properties.get("key3"), Some(&json!("new_key")));

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_register_once_no_default() {
        let (persistence, file_path) = setup_test_persistence("register_once_nodefault");
        let mut initial_props = HashMap::new();
        initial_props.insert("key1".to_string(), json!("initial"));
        persistence.register(initial_props, None);
        wait_for_save().await;

        let mut new_props = HashMap::new();
        new_props.insert("key1".to_string(), json!("new_value")); // Should not overwrite
        new_props.insert("key2".to_string(), json!("new_key")); // Should add

        persistence.register_once(new_props, None, None);
        wait_for_save().await;

        let props = persistence.get_properties();
        assert_eq!(props.get("key1"), Some(&json!("initial"))); // Remains initial
        assert_eq!(props.get("key2"), Some(&json!("new_key"))); // Added

        let file_data = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data.properties.get("key1"), Some(&json!("initial")));
        assert_eq!(file_data.properties.get("key2"), Some(&json!("new_key")));

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_unregister() {
        let (persistence, file_path) = setup_test_persistence("unregister");
        let mut props = HashMap::new();
        props.insert("key_to_keep".to_string(), json!("keep"));
        props.insert("key_to_remove".to_string(), json!("remove"));
        persistence.register(props, None);
        wait_for_save().await;

        persistence.unregister("key_to_remove");
        wait_for_save().await;

        let current_props = persistence.get_properties();
        assert_eq!(current_props.len(), 1);
        assert!(current_props.contains_key("key_to_keep"));
        assert!(!current_props.contains_key("key_to_remove"));

        let file_data = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data.properties.len(), 1);
        assert!(file_data.properties.contains_key("key_to_keep"));

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_set_get_distinct_id() {
        let (persistence, file_path) = setup_test_persistence("distinct_id");

        assert_eq!(persistence.get_distinct_id(), None);
        persistence.set_distinct_id(Some("user123".to_string()));
        wait_for_save().await;

        assert_eq!(persistence.get_distinct_id(), Some("user123".to_string()));

        let file_data = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data.distinct_id, Some("user123".to_string()));

        persistence.set_distinct_id(None);
        wait_for_save().await;
        assert_eq!(persistence.get_distinct_id(), None);

        let file_data_cleared = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data_cleared.distinct_id, None);

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_event_timers() {
        let (persistence, file_path) = setup_test_persistence("event_timers");
        let now = current_time_millis();
        persistence.set_event_timer("test_event".to_string(), now);
        wait_for_save().await;

        let file_data = read_test_file(&file_path).await.unwrap();
        assert_eq!(file_data.event_timers.get("test_event"), Some(&now));

        let removed_time = persistence.remove_event_timer("test_event");
        assert_eq!(removed_time, Some(now));
        wait_for_save().await;

        let file_data_after_remove = read_test_file(&file_path).await.unwrap();
        assert!(file_data_after_remove.event_timers.is_empty());
        assert!(persistence.remove_event_timer("test_event").is_none()); // Already removed

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_clear_all_data() {
        let (persistence, file_path) = setup_test_persistence("clear_all");
        persistence.set_distinct_id(Some("user_clear".to_string()));
        let mut props = HashMap::new();
        props.insert("prop".to_string(), json!("value"));
        persistence.register(props, None);
        persistence.set_event_timer("timer".to_string(), 12345);
        wait_for_save().await;

        assert!(read_test_file(&file_path)
            .await
            .unwrap()
            .distinct_id
            .is_some());

        persistence.clear_all_data();
        wait_for_save().await;

        assert!(persistence.get_distinct_id().is_none());
        assert!(persistence.get_properties().is_empty());
        assert!(persistence.remove_event_timer("timer").is_none());

        let file_data = read_test_file(&file_path).await.unwrap();
        assert!(file_data.distinct_id.is_none());
        assert!(file_data.properties.is_empty());
        assert!(file_data.event_timers.is_empty());
        assert!(file_data.store_expires_at.is_none());

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_register_with_expiration() {
        let (persistence, file_path) = setup_test_persistence("register_expiry");
        let mut props = HashMap::new();
        props.insert("temp_prop".to_string(), json!("expires_soon"));

        persistence.register(props.clone(), Some(0));
        wait_for_save().await;
        let file_data = read_test_file(&file_path).await.unwrap();
        assert!(
            file_data.store_expires_at.is_none(),
            "Expiration should be None for 0 days"
        );
        assert!(
            persistence.get_properties().contains_key("temp_prop"),
            "Property should exist immediately after register with 0 days"
        );

        persistence.register(props, Some(1));
        wait_for_save().await;
        let file_data_1_day = read_test_file(&file_path).await.unwrap();
        assert!(
            file_data_1_day.store_expires_at.is_some(),
            "Expiration should be set for 1 day"
        );
        assert!(
            persistence.get_properties().contains_key("temp_prop"),
            "Property should exist immediately after register with 1 day"
        );

        cleanup_test_file(&file_path);
    }

    #[tokio::test]
    async fn test_properties_respect_expiration() {
        let (persistence, file_path) = setup_test_persistence("prop_expiry");
        let mut props = HashMap::new();
        props.insert("prop1".to_string(), json!("value1"));
        persistence.register(props, None); // Register without expiry first
        wait_for_save().await;

        let now = current_time_millis();
        let expired_data = PersistentData {
            properties: {
                let mut p = HashMap::new();
                p.insert("expired_prop".to_string(), json!(true));
                p
            },
            store_expires_at: Some(now - 1000), // 1 second in the past
            ..Default::default()
        };
        persistence.write_data_async(expired_data).await.unwrap();
        wait_for_save().await; // Ensure write completes

        let persistence_reloaded = Persistence::new(&file_path);

        assert!(
            persistence_reloaded.get_properties().is_empty(),
            "Properties should be empty after loading expired data"
        );

        cleanup_test_file(&file_path);
    }

    #[test]
    fn test_register_options_parsing() {
        // persistent: true (default), days: None (default)
        let options_none = None;
        let parsed_none = RegisterOptions::parse_options(options_none);
        assert_eq!(parsed_none.persistent, true);
        assert_eq!(parsed_none.days, None);

        // persistent: true (default), days: None (explicit null)
        let options_null = Some(json!({"days": null}));
        let parsed_null = RegisterOptions::parse_options(options_null);
        assert_eq!(parsed_null.persistent, true);
        assert_eq!(parsed_null.days, None);

        // persistent: true (default), days: 10
        let options_days = Some(json!({"days": 10}));
        let parsed_days = RegisterOptions::parse_options(options_days);
        assert_eq!(parsed_days.persistent, true);
        assert_eq!(parsed_days.days, Some(10));

        // persistent: false, days: None (default)
        let options_not_persistent = Some(json!({"persistent": false}));
        let parsed_not_persistent = RegisterOptions::parse_options(options_not_persistent);
        assert_eq!(parsed_not_persistent.persistent, false);
        assert_eq!(parsed_not_persistent.days, None);

        // persistent: false, days: 5
        let options_both = Some(json!({"persistent": false, "days": 5}));
        let parsed_both = RegisterOptions::parse_options(options_both);
        assert_eq!(parsed_both.persistent, false);
        assert_eq!(parsed_both.days, Some(5));

        // Extra properties ignored
        let options_extra = Some(json!({"persistent": false, "extra": "ignored"}));
        let parsed_extra = RegisterOptions::parse_options(options_extra);
        assert_eq!(parsed_extra.persistent, false);
        assert_eq!(parsed_extra.days, None);

        // Invalid types default
        let options_invalid = Some(json!({"persistent": "not a bool", "days": "not a number"}));
        let parsed_invalid = RegisterOptions::parse_options(options_invalid);
        assert_eq!(parsed_invalid.persistent, true); // defaults to true
        assert_eq!(parsed_invalid.days, None);

        // Not an object defaults
        let options_not_object = Some(json!(["persistent", false]));
        let parsed_not_object = RegisterOptions::parse_options(options_not_object);
        assert_eq!(parsed_not_object.persistent, true);
        assert_eq!(parsed_not_object.days, None);
    }
}
