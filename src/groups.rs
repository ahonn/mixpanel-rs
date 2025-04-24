use crate::{Mixpanel, Modifiers, Result};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct MixpanelGroups {
    pub(crate) mixpanel: Option<Box<Mixpanel>>,
}

impl MixpanelGroups {
    /// Set properties on a group profile
    pub async fn set<S: Into<String>>(
        &self,
        group_key: S,
        group_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        self._set(
            group_key.into(),
            group_id.into(),
            properties,
            modifiers,
            false,
        )
        .await
    }

    /// Set properties on a group profile only if they haven't been set before
    pub async fn set_once<S: Into<String>>(
        &self,
        group_key: S,
        group_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        self._set(
            group_key.into(),
            group_id.into(),
            properties,
            modifiers,
            true,
        )
        .await
    }

    /// Delete a group profile
    pub async fn delete_group<S: Into<String>>(
        &self,
        group_key: S,
        group_id: S,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$group_key": group_key.into(),
            "$group_id": group_id.into(),
            "$delete": ""
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/groups", &data)
            .await
    }

    /// Remove a value from a list-valued group profile property
    pub async fn remove<S: Into<String>>(
        &self,
        group_key: S,
        group_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$group_key": group_key.into(),
            "$group_id": group_id.into(),
            "$remove": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/groups", &data)
            .await
    }

    /// Union a value to a list-valued group profile property
    pub async fn union<S: Into<String>>(
        &self,
        group_key: S,
        group_id: S,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$group_key": group_key.into(),
            "$group_id": group_id.into(),
            "$union": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/groups", &data)
            .await
    }

    /// Unset properties on a group profile
    pub async fn unset<S: Into<String>>(
        &self,
        group_key: S,
        group_id: S,
        properties: Vec<String>,
        modifiers: Option<Modifiers>,
    ) -> Result<()> {
        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$group_key": group_key.into(),
            "$group_id": group_id.into(),
            "$unset": properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/groups", &data)
            .await
    }

    // Internal helper for set and set_once
    async fn _set(
        &self,
        group_key: String,
        group_id: String,
        properties: HashMap<String, Value>,
        modifiers: Option<Modifiers>,
        set_once: bool,
    ) -> Result<()> {
        let operation = if set_once { "$set_once" } else { "$set" };

        let mut data = serde_json::json!({
            "$token": self.mixpanel.as_ref().unwrap().token,
            "$group_key": group_key,
            "$group_id": group_id,
            operation: properties
        });

        if let Some(modifiers) = modifiers {
            data = crate::utils::merge_modifiers(data, Some(modifiers));
        }

        self.mixpanel
            .as_ref()
            .unwrap()
            .send_request("GET", "/groups", &data)
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

        let result = mp.groups.set("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_set_once() {
        let mp = Mixpanel::init("test_token", None);
        let mut props = HashMap::new();
        props.insert("key1".to_string(), "value1".into());

        let result = mp.groups.set_once("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_group() {
        let mp = Mixpanel::init("test_token", None);
        let result = mp.groups.delete_group("company", "Acme Inc", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        props.insert("products".to_string(), "anvil".into());

        let result = mp.groups.remove("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_remove_multiple() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        props.insert("products".to_string(), "anvil".into());
        props.insert("customer_segments".to_string(), "coyotes".into());

        let result = mp.groups.remove("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_union() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();
        let products: Vec<Value> = vec!["anvil".into()];
        props.insert(
            "products".to_string(),
            serde_json::to_value(products).unwrap(),
        );

        let result = mp.groups.union("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_union_multiple() {
        let mp = Mixpanel::init("test_token", None);

        let mut props = HashMap::new();

        let products: Vec<Value> = vec!["anvil".into()];
        props.insert(
            "products".to_string(),
            serde_json::to_value(products).unwrap(),
        );

        let segments: Vec<Value> = vec!["coyotes".into()];
        props.insert(
            "customer_segments".to_string(),
            serde_json::to_value(segments).unwrap(),
        );

        let result = mp.groups.union("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unset() {
        let mp = Mixpanel::init("test_token", None);

        let props = vec!["products".to_string()];

        let result = mp.groups.unset("company", "Acme Inc", props, None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_unset_multiple() {
        let mp = Mixpanel::init("test_token", None);

        let props = vec!["products".to_string(), "customer_segments".to_string()];

        let result = mp.groups.unset("company", "Acme Inc", props, None).await;
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

        let result = mp
            .groups
            .set("company", "Acme Inc", props, Some(modifiers))
            .await;
        assert!(result.is_ok());
    }
}
