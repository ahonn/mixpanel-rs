// mixpanel-rs: A Rust client for Mixpanel
//
// Inspired by the Node.js library (https://github.com/mixpanel/mixpanel-node)

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use groups::MixpanelGroups;
use people::MixpanelPeople;
use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time;
use error::Error;

pub mod error;
pub mod groups;
pub mod people;
mod utils;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub test: bool,
    pub debug: bool,
    pub verbose: bool,
    pub host: String,
    pub protocol: String,
    pub path: String,
    pub secret: Option<String>,
    pub api_key: Option<String>,
    pub geolocate: bool,
    pub max_retries: u32,
    pub retry_base_delay_ms: u64,
    pub retry_max_delay_ms: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            test: false,
            debug: false,
            verbose: false,
            host: "api.mixpanel.com".to_string(),
            protocol: "https".to_string(),
            path: "".to_string(),
            secret: None,
            api_key: None,
            geolocate: false,
            max_retries: 3,
            retry_base_delay_ms: 1000,
            retry_max_delay_ms: 10000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Modifiers {
    #[serde(rename = "$ip", skip_serializing_if = "Option::is_none")]
    pub ip: Option<String>,

    #[serde(rename = "$ignore_time", skip_serializing_if = "Option::is_none")]
    pub ignore_time: Option<bool>,

    #[serde(rename = "$time", skip_serializing_if = "Option::is_none")]
    pub time: Option<u64>,

    #[serde(rename = "$ignore_alias", skip_serializing_if = "Option::is_none")]
    pub ignore_alias: Option<bool>,

    #[serde(rename = "$latitude", skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,

    #[serde(rename = "$longitude", skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Mixpanel {
    pub token: String,
    pub config: Config,
    pub people: MixpanelPeople,
    pub groups: MixpanelGroups,
    http_client: Client,
}

impl Mixpanel {
    /// Initialize a new Mixpanel client with the given token and optional config
    pub fn init(token: &str, config: Option<Config>) -> Self {
        let config = config.unwrap_or_default();
        let http_client = Client::builder()
            .build()
            .expect("Failed to create HTTP client");

        let mut instance = Self {
            token: token.to_string(),
            config,
            people: MixpanelPeople::default(),
            groups: MixpanelGroups::default(),
            http_client,
        };

        instance.people.mixpanel = Some(Box::new(instance.clone()));
        instance.groups.mixpanel = Some(Box::new(instance.clone()));

        instance
    }

    /// Track an event with optional properties
    pub async fn track<S: Into<String>>(
        &self,
        event: S,
        properties: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<()> {
        let mut props = properties.unwrap_or_default();
        props.insert("token".to_string(), self.token.clone().into());
        props.insert("mp_lib".to_string(), "rust".into());
        props.insert("$lib_version".to_string(), env!("CARGO_PKG_VERSION").into());

        // Handle time property if it exists
        if let Some(time_value) = props.get("time") {
            if let Some(time_num) = time_value.as_u64() {
                props.insert("time".to_string(), time_num.into());
            } else if let Some(time_str) = time_value.as_str() {
                // Try to parse as ISO string - simplified for now
                if let Ok(time_num) = time_str.parse::<u64>() {
                    props.insert("time".to_string(), time_num.into());
                }
            }
        }

        let data = Event {
            event: event.into(),
            properties: props,
        };

        if self.config.debug {
            println!("Sending event to Mixpanel: {:?}", &data);
        }

        self.send_request("GET", "/track", &data).await
    }

    /// Track multiple events in a single request (batch)
    pub async fn track_batch(&self, events: Vec<Event>) -> Result<()> {
        // Process each event to ensure it has the required properties
        let events: Vec<Event> = events
            .into_iter()
            .map(|event| {
                let mut props = event.properties;
                props.insert("token".to_string(), self.token.clone().into());
                props.insert("mp_lib".to_string(), "rust".into());
                props.insert("$lib_version".to_string(), env!("CARGO_PKG_VERSION").into());

                Event {
                    event: event.event,
                    properties: props,
                }
            })
            .collect();

        if self.config.debug {
            println!("Sending batch of {} events to Mixpanel", events.len());
        }

        // Mixpanel accepts a maximum of 50 events per request
        const MAX_BATCH_SIZE: usize = 50;

        for chunk in events.chunks(MAX_BATCH_SIZE) {
            self.send_request("POST", "/track", chunk).await?;
        }

        Ok(())
    }

    /// Create an alias for a distinct_id
    pub async fn alias<S: Into<String>>(&self, distinct_id: S, alias: S) -> Result<()> {
        let mut properties = HashMap::new();
        properties.insert("distinct_id".to_string(), distinct_id.into().into());
        properties.insert("alias".to_string(), alias.into().into());

        self.track("$create_alias", Some(properties)).await
    }

    /// Send a request to the Mixpanel API with automatic retries for certain error types
    pub async fn send_request<T: Serialize + ?Sized>(
        &self,
        method: &str,
        endpoint: &str,
        data: &T,
    ) -> Result<()> {
        let mut retries = 0;
        let max_retries = self.config.max_retries;
        
        loop {
            match self.do_send_request(method, endpoint, data).await {
                Ok(result) => return Ok(result),
                
                Err(err) => {
                    if retries >= max_retries {
                        return Err(Error::MaxRetriesReached(format!(
                            "Failed after {} retries. Last error: {}", 
                            retries, err
                        )));
                    }
                    
                    let should_retry = match &err {
                        Error::HttpError(http_err) => http_err.is_connect() || http_err.is_timeout(),
                        Error::ApiServerError(_) => true,
                        Error::ApiRateLimitError(_) => true,
                        _ => false,
                    };
                    
                    if !should_retry {
                        return Err(err);
                    }
                    
                    let base_delay = self.config.retry_base_delay_ms;
                    let max_delay = self.config.retry_max_delay_ms;
                    
                    let wait_time = match &err {
                        Error::ApiRateLimitError(Some(retry_after)) => {
                            Duration::from_secs(*retry_after)
                        },
                        _ => {
                            let delay = base_delay * (1 << retries);
                            let capped_delay = std::cmp::min(delay, max_delay);
                            Duration::from_millis(capped_delay)
                        }
                    };
                    
                    if self.config.debug {
                        println!("Retrying request after error: {}. Retry {} of {}. Waiting {:?}", 
                                 err, retries + 1, max_retries, wait_time);
                    }
                    
                    time::sleep(wait_time).await;
                    retries += 1;
                }
            }
        }
    }

    /// Internal method to send a request without retries
    async fn do_send_request<T: Serialize + ?Sized>(
        &self,
        method: &str,
        endpoint: &str,
        data: &T,
    ) -> Result<()> {
        let data_json = serde_json::to_string(data)?;
        let encoded_data = BASE64.encode(data_json.as_bytes());

        let mut url = Url::parse(&format!(
            "{}://{}{}",
            self.config.protocol, self.config.host, self.config.path
        ))?;

        let endpoint = if endpoint.starts_with('/') {
            &endpoint[1..]
        } else {
            endpoint
        };
        url.set_path(&format!("{}{}", url.path(), endpoint));

        {
            let mut query_pairs = url.query_pairs_mut();

            if self.config.geolocate {
                query_pairs.append_pair("ip", "1");
            } else {
                query_pairs.append_pair("ip", "0");
            }

            if self.config.verbose {
                query_pairs.append_pair("verbose", "1");
            } else {
                query_pairs.append_pair("verbose", "0");
            }

            if method.to_uppercase() == "GET" {
                query_pairs.append_pair("data", &encoded_data);
            }

            if self.config.test {
                query_pairs.append_pair("test", "1");
            }
        }

        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => {
                let mut builder = self.http_client.post(url);
                builder = builder.header("Content-Type", "application/x-www-form-urlencoded");
                builder = builder.body(format!("data={}", encoded_data));
                builder
            }
            _ => {
                return Err(Error::ApiClientError(
                    0,
                    format!("Unsupported HTTP method: {}", method),
                ));
            }
        };

        if let Some(ref secret) = self.config.secret {
            let auth_header = format!("Basic {}", BASE64.encode(format!("{}:", secret).as_bytes()));
            request_builder = request_builder.header("Authorization", auth_header);
        }

        let response = request_builder.send().await?;
        let status = response.status();
        let status_code = status.as_u16();

        if status.is_success() {
            let body = response.text().await?;
            if self.config.verbose {
                match serde_json::from_str::<serde_json::Value>(&body) {
                    Ok(json) => {
                        if let Some(api_status) = json.get("status").and_then(|s| s.as_u64()) {
                            if api_status != 1 {
                                if let Some(error_msg) = json.get("error").and_then(|e| e.as_str())
                                {
                                    return Err(Error::ApiClientError(
                                        status_code,
                                        error_msg.to_string(),
                                    ));
                                } else {
                                    return Err(Error::ApiUnexpectedResponse(format!(
                                        "Response status was not 1: {}",
                                        body
                                    )));
                                }
                            }
                            Ok(())
                        } else {
                            Err(Error::ApiUnexpectedResponse(format!(
                                "Response missing status: {}",
                                body
                            )))
                        }
                    }
                    Err(e) => Err(Error::JsonError(e)),
                }
            } else if body != "1" {
                Err(Error::ApiUnexpectedResponse(body))
            } else {
                Ok(())
            }
        } else {
            match status_code {
                413 => Err(Error::ApiPayloadTooLarge),
                429 => {
                    let retry_after = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    Err(Error::ApiRateLimitError(retry_after))
                }
                s if s >= 500 => Err(Error::ApiServerError(s)),
                s if s >= 400 => {
                    let body = response.text().await.unwrap_or_else(|e| e.to_string());
                    Err(Error::ApiClientError(s, body))
                }
                _ => {
                    let body = response.text().await.unwrap_or_else(|e| e.to_string());
                    Err(Error::ApiHttpError(status_code, body))
                }
            }
        }
    }

    pub fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        let mp = Mixpanel::init("test_token", None);
        assert_eq!(mp.token, "test_token");
        assert_eq!(mp.config.host, "api.mixpanel.com");
    }

    #[test]
    fn test_custom_config() {
        let config = Config {
            host: "custom.example.com".to_string(),
            test: true,
            ..Default::default()
        };

        let mp = Mixpanel::init("test_token", Some(config));
        assert_eq!(mp.config.host, "custom.example.com");
        assert!(mp.config.test);
    }
}
