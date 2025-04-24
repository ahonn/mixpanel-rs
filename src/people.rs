use crate::{Mixpanel, Modifiers, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MixpanelPeople {
    pub(crate) mixpanel: Option<Box<Mixpanel>>,
}

impl MixpanelPeople {
    /// Set properties on a user profile
    pub async fn set<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        self._set(distinct_id.into(), properties, modifiers, false)
            .await
    }

    /// Set properties on a user profile only if they haven't been set before
    pub async fn set_once<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        self._set(distinct_id.into(), properties, modifiers, true)
            .await
    }

    /// Increment numeric properties on a user profile
    pub async fn increment<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: HashMap<String, i64>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$add": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Append values to list properties on a user profile
    pub async fn append<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$append": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Track a charge on a user profile
    pub async fn track_charge<S: Into<String>>(
        &self,
        distinct_id: S,
        amount: f64,
        properties: Option<HashMap<String, Value>>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut charge = properties.unwrap_or_default();
        charge.insert("$amount".to_string(), amount.into());

        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$append": {
                "$transactions": charge
            }
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Clear all charges from a user profile
    pub async fn clear_charges<S: Into<String>>(
        &self,
        distinct_id: S,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$set": {
                "$transactions": []
            }
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Delete a user profile
    pub async fn delete_user<S: Into<String>>(
        &self,
        distinct_id: S,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$delete": ""
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Remove values from list properties on a user profile
    pub async fn remove<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$remove": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Union values to list properties on a user profile
    pub async fn union<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$union": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    /// Unset properties on a user profile
    pub async fn unset<S: Into<String>>(
        &self,
        distinct_id: S,
        properties: Vec<String>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id.into(),
            "$unset": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }

    // Internal helper for set and set_once
    async fn _set(
        &self,
        distinct_id: String,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
        set_once: bool,
    ) -> Result<()> {
        let operation = if set_once { "$set_once" } else { "$set" };

        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$distinct_id": distinct_id,
            operation: properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/engage", &data)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_set() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("key1".to_string(), "value1".into());

        let result = mp.people.set("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_once() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("key1".to_string(), "value1".into());

        let result = mp.people.set_once("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_increment() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("counter".to_string(), 1);

        let result = mp.people.increment("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_increment_with_value() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("counter".to_string(), 5);
        props.insert("another_counter".to_string(), -3);

        let result = mp.people.increment("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_append() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("items".to_string(), "item1".into());

        let result = mp.people.append("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_append_multiple() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("items".to_string(), "item1".into());
        props.insert("tags".to_string(), "tag1".into());

        let result = mp.people.append("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_track_charge() {
        let mp = Mixpanel::init("test_token", None);

        let result = mp.people.track_charge("test_user", 50.0, None, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_track_charge_with_properties() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        props.insert("item".to_string(), "Premium Plan".into());

        let result = mp
            .people
            .track_charge("test_user", 50.0, Some(props), None)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_clear_charges() {
        let mp = Mixpanel::init("test_token", None);

        let result = mp.people.clear_charges("test_user", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let mp = Mixpanel::init("test_token", None);

        let result = mp.people.delete_user("test_user", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_user_with_modifiers() {
        let mp = Mixpanel::init("test_token", None);

        let modifiers = Modifiers {
            ignore_time: Some(true),
            ignore_alias: Some(true),
            ..Default::default()
        };

        let result = mp.people.delete_user("test_user", Some(modifiers)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        props.insert("browsers".to_string(), "firefox".into());

        let result = mp.people.remove("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_multiple() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        props.insert("browsers".to_string(), "firefox".into());
        props.insert("apps".to_string(), "vscode".into());

        let result = mp.people.remove("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_union() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        let browsers: Vec<Value> = vec!["firefox".into(), "chrome".into()];
        props.insert(
            "browsers".to_string(),
            serde_json::to_value(browsers).unwrap(),
        );

        let result = mp.people.union("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_union_scalar() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        // 创建一个只包含一个元素的数组，而不是标量值
        let browsers: Vec<Value> = vec!["firefox".into()];
        props.insert(
            "browsers".to_string(),
            serde_json::to_value(browsers).unwrap(),
        );

        let result = mp.people.union("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unset() {
        let mp = Mixpanel::init("test_token", None);

        let props = vec!["key1".to_string()];

        let result = mp.people.unset("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unset_multiple() {
        let mp = Mixpanel::init("test_token", None);

        let props = vec!["key1".to_string(), "key2".to_string()];

        let result = mp.people.unset("test_user", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_with_modifiers() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        props.insert("key1".to_string(), "value1".into());

        let modifiers = Modifiers {
            ip: Some("1.2.3.4".to_string()),
            ignore_time: Some(true),
            ..Default::default()
        };

        let result = mp.people.set("test_user", props, Some(modifiers)).await;
        assert!(result.is_ok());
    }
}

