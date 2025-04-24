use crate::error::{Error, Result};
use mixpanel_rs::{Config, Mixpanel};
use serde_json::Value;
use std::collections::HashMap;

pub struct MixpanelState {
    client: Mixpanel,
}

impl MixpanelState {
    pub fn new(token: String, api_host: Option<String>) -> Self {
        let config = Config {
            host: api_host.unwrap_or_else(|| "api.mixpanel.com".to_string()),
            ..Default::default()
        };

        let client = Mixpanel::init(&token, Some(config));

        Self { client }
    }

    pub async fn track(&self, event_name: &str, properties: Option<Value>) -> Result<()> {
        if self.client.config.test {
            return Ok(());
        }

        let props = properties.and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>()
            })
        });

        self.client
            .track(event_name, props)
            .await
            .map_err(|e| Error::MixpanelError(e.to_string()))
    }

    pub async fn identify(&self, distinct_id: &str, properties: Option<Value>) -> Result<()> {
        if self.client.config.test {
            return Ok(());
        }

        let props = properties.and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<HashMap<_, _>>()
            })
        });

        self.client
            .people
            .set(distinct_id, props.unwrap_or_default(), None)
            .await
            .map_err(|e| Error::MixpanelError(e.to_string()))
    }

    pub async fn alias(&self, distinct_id: &str, alias: &str) -> Result<()> {
        if self.client.config.test {
            return Ok(());
        }

        self.client
            .alias(distinct_id, alias)
            .await
            .map_err(|e| Error::MixpanelError(e.to_string()))
    }
}

