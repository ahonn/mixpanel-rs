use crate::error::{Error, Result};
use crate::persistence::Persistence;
use mixpanel_rs::Mixpanel;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub(crate) const SET_ACTION: &str = "$set";
pub(crate) const SET_ONCE_ACTION: &str = "$set_once";
pub(crate) const UNSET_ACTION: &str = "$unset";
pub(crate) const ADD_ACTION: &str = "$add";
pub(crate) const APPEND_ACTION: &str = "$append";
pub(crate) const REMOVE_ACTION: &str = "$remove";
pub(crate) const UNION_ACTION: &str = "$union";
pub(crate) const DELETE_ACTION: &str = "$delete";

pub struct MixpanelPeople {
    client: Mixpanel,
    persistence: Arc<Persistence>,
}

impl MixpanelPeople {
    pub(crate) fn new(client: Mixpanel, persistence: Arc<Persistence>) -> Self {
        Self {
            client,
            persistence,
        }
    }

    fn get_distinct_id(&self) -> Option<String> {
        self.persistence.get_distinct_id()
    }

    fn identify_called(&self) -> bool {
        self.get_distinct_id()
            .map_or(false, |id| !id.starts_with("$device:"))
    }

    fn is_reserved_property(&self, prop: &str) -> bool {
        matches!(
            prop,
            "$distinct_id" | "$token" | "$device_id" | "$user_id" | "$had_persisted_distinct_id"
        )
    }

    /// Internal function to prepare and send the people request.
    async fn send_request(&self, action: &str, properties: HashMap<String, Value>) -> Result<()> {
        if !self.identify_called() {
            println!("Mixpanel People: identify() must be called before using People API methods. Operation queued (in theory - queuing not fully implemented yet).");
            return Ok(());
        }

        let distinct_id = self.get_distinct_id().ok_or_else(|| {
            Error::MixpanelError(
                "Cannot perform People operation without a distinct_id.".to_string(),
            )
        })?;
        let map_err = |e: mixpanel_rs::error::Error| Error::MixpanelClient(e);

        match action {
            SET_ACTION => self
                .client
                .people
                .set(&distinct_id, properties, None)
                .await
                .map_err(map_err)?,
            SET_ONCE_ACTION => self
                .client
                .people
                .set_once(&distinct_id, properties, None)
                .await
                .map_err(map_err)?,
            UNSET_ACTION => {
                let keys_to_unset: Vec<String> = properties.keys().cloned().collect();
                self.client
                    .people
                    .unset(&distinct_id, keys_to_unset, None)
                    .await
                    .map_err(map_err)?
            }
            ADD_ACTION => {
                let mut increment_props: HashMap<String, i64> = HashMap::new();
                for (key, value) in properties {
                    if let Some(num) = value.as_i64() {
                        increment_props.insert(key, num);
                    } else {
                        eprintln!(
                            "Mixpanel People: Invalid increment value for key '{}' - must be convertible to i64.",
                            key
                        );
                        return Err(Error::MixpanelError(format!(
                            "Invalid increment value for key '{}'",
                            key
                        )));
                    }
                }

                self.client
                    .people
                    .increment(&distinct_id, increment_props, None)
                    .await
                    .map_err(map_err)?
            }
            APPEND_ACTION => self
                .client
                .people
                .append(&distinct_id, properties, None)
                .await
                .map_err(map_err)?,
            REMOVE_ACTION => self
                .client
                .people
                .remove(&distinct_id, properties, None)
                .await
                .map_err(map_err)?,
            UNION_ACTION => self
                .client
                .people
                .union(&distinct_id, properties, None)
                .await
                .map_err(map_err)?,
            DELETE_ACTION => self
                .client
                .people
                .delete_user(&distinct_id, None)
                .await
                .map_err(map_err)?,
            _ => {
                return Err(Error::MixpanelError(format!(
                    "Unknown People action: {}",
                    action
                )))
            }
        };

        Ok(())
    }

    /// Parses a serde_json::Value into a HashMap<String, Value>.
    /// Filters out reserved properties.
    fn parse_and_filter_props(
        &self,
        value: Value,
        action_name: &str,
    ) -> Result<HashMap<String, Value>> {
        match value {
            Value::Object(obj) => Ok(obj
                .into_iter()
                .filter(|(k, _)| !self.is_reserved_property(k))
                .collect()),
            Value::Null => Ok(HashMap::new()),
            _ => Err(Error::MixpanelError(format!(
                "Properties for people.{} must be an object or null",
                action_name
            ))),
        }
    }

    /// Set properties on a user profile.
    ///
    /// If `prop` is a String, `to` is the value.
    /// If `prop` is an Object (Value::Object), `to` is ignored and the object keys/values are used.
    pub async fn set(&self, prop: Value, to: Option<Value>) -> Result<()> {
        let properties = match prop {
            Value::Object(map) => self.parse_and_filter_props(Value::Object(map), "set")?,
            Value::String(key) => {
                if self.is_reserved_property(&key) {
                    return Ok(());
                }
                let mut map = HashMap::new();
                map.insert(key, to.unwrap_or(Value::Null));
                map
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'prop' argument for people.set. Must be String or Object.".to_string(),
                ))
            }
        };

        if properties.is_empty() {
            return Ok(());
        }
        self.send_request(SET_ACTION, properties).await
    }

    /// Set properties on a user profile, only if they do not yet exist.
    pub async fn set_once(&self, prop: Value, to: Option<Value>) -> Result<()> {
        let properties = match prop {
            Value::Object(map) => self.parse_and_filter_props(Value::Object(map), "set_once")?,
            Value::String(key) => {
                if self.is_reserved_property(&key) {
                    return Ok(());
                }
                let mut map = HashMap::new();
                map.insert(key, to.unwrap_or(Value::Null));
                map
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'prop' argument for people.set_once. Must be String or Object."
                        .to_string(),
                ))
            }
        };

        if properties.is_empty() {
            return Ok(());
        }

        self.send_request(SET_ONCE_ACTION, properties).await
    }

    /// Unset properties on a user profile.
    ///
    /// `prop` should be a String (single key) or an Array of Strings (multiple keys).
    pub async fn unset(&self, prop: Value) -> Result<()> {
        let mut keys_to_unset = HashMap::new();

        match prop {
            Value::String(key) => {
                if !self.is_reserved_property(&key) {
                    keys_to_unset.insert(key, Value::Null);
                }
            }
            Value::Array(keys) => {
                for key_val in keys {
                    if let Value::String(key) = key_val {
                        if !self.is_reserved_property(&key) {
                            keys_to_unset.insert(key, Value::Null);
                        }
                    } else {
                        return Err(Error::MixpanelError(
                            "Invalid array element in people.unset. Must be strings.".to_string(),
                        ));
                    }
                }
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'prop' argument for people.unset. Must be String or Array of Strings."
                        .to_string(),
                ))
            }
        }

        if keys_to_unset.is_empty() {
            return Ok(());
        }

        self.send_request(UNSET_ACTION, keys_to_unset).await
    }

    /// Increment/decrement numeric user profile properties.
    ///
    /// If `prop` is a String, `by` is the numeric amount.
    /// If `prop` is an Object (Value::Object), `by` is ignored, and the object's key/numeric values are used.
    pub async fn increment(&self, prop: Value, by: Option<Value>) -> Result<()> {
        let properties = match prop {
            Value::Object(map) => {
                let mut filtered_map = HashMap::new();
                for (key, value) in map {
                    if self.is_reserved_property(&key) {
                        continue;
                    }
                    if value.is_number() {
                        filtered_map.insert(key, value);
                    } else {
                        eprintln!(
                            "Mixpanel People: Invalid increment value for key '{}' - must be a number.",
                            key
                        );
                        return Err(Error::MixpanelError(format!(
                            "Invalid increment value for key '{}'",
                            key
                        )));
                    }
                }
                filtered_map
            }
            Value::String(key) => {
                if self.is_reserved_property(&key) {
                    return Ok(());
                }
                let amount = by.unwrap_or(Value::Number(1.into())); // Default increment by 1
                if !amount.is_number() {
                    return Err(Error::MixpanelError(
                        "Invalid 'by' argument for people.increment. Must be a number.".to_string(),
                    ));
                }
                let mut map = HashMap::new();
                map.insert(key, amount);
                map
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'prop' argument for people.increment. Must be String or Object."
                        .to_string(),
                ))
            }
        };

        if properties.is_empty() {
            return Ok(());
        }

        self.send_request(ADD_ACTION, properties).await
    }

    /// Append a value to a list-valued user profile property.
    pub async fn append(&self, list_name: Value, value: Option<Value>) -> Result<()> {
        let properties = match list_name {
            Value::Object(map) => self.parse_and_filter_props(Value::Object(map), "append")?,
            Value::String(key) => {
                if self.is_reserved_property(&key) {
                    return Ok(());
                }
                let mut map = HashMap::new();
                map.insert(key, value.unwrap_or(Value::Null));
                map
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'list_name' argument for people.append. Must be String or Object."
                        .to_string(),
                ))
            }
        };

        if properties.is_empty() {
            return Ok(());
        }

        self.send_request(APPEND_ACTION, properties).await
    }

    /// Remove a value from a list-valued user profile property.
    pub async fn remove(&self, list_name: Value, value: Option<Value>) -> Result<()> {
        let properties = match list_name {
            Value::Object(map) => self.parse_and_filter_props(Value::Object(map), "remove")?,
            Value::String(key) => {
                if self.is_reserved_property(&key) {
                    return Ok(());
                }
                let mut map = HashMap::new();
                map.insert(key, value.unwrap_or(Value::Null));
                map
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'list_name' argument for people.remove. Must be String or Object."
                        .to_string(),
                ))
            }
        };

        if properties.is_empty() {
            return Ok(());
        }

        self.send_request(REMOVE_ACTION, properties).await
    }

    /// Merge a list with a list-valued user profile property, excluding duplicate values.
    pub async fn union(&self, list_name: Value, values: Option<Value>) -> Result<()> {
        let properties = match list_name {
            Value::Object(map) => {
                let mut processed_map = HashMap::new();
                for (key, value) in map {
                    if self.is_reserved_property(&key) {
                        continue;
                    }
                    match value {
                        Value::Array(arr) => {
                            processed_map.insert(key, Value::Array(arr));
                        }
                        single_val => {
                            processed_map.insert(key, Value::Array(vec![single_val]));
                        }
                    }
                }
                processed_map
            }
            Value::String(key) => {
                if self.is_reserved_property(&key) {
                    return Ok(());
                }
                let values_to_union = match values.unwrap_or(Value::Array(vec![])) {
                    Value::Array(arr) => Value::Array(arr),
                    single_val => Value::Array(vec![single_val]),
                };
                let mut map = HashMap::new();
                map.insert(key, values_to_union);
                map
            }
            _ => {
                return Err(Error::MixpanelError(
                    "Invalid 'list_name' argument for people.union. Must be String or Object."
                        .to_string(),
                ))
            }
        };

        if properties.is_empty() {
            return Ok(());
        }

        self.send_request(UNION_ACTION, properties).await
    }

    /// Permanently delete the user's profile.
    pub async fn delete_user(&self) -> Result<()> {
        if !self.identify_called() {
            eprintln!("Mixpanel People: delete_user() requires identify() to be called first.");
            return Ok(());
        }
        self.send_request(DELETE_ACTION, HashMap::new()).await
    }
}
