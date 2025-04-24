// mixpanel-rs: A Rust client for Mixpanel
//
// Inspired by the Node.js library (https://github.com/mixpanel/mixpanel-node)

use reqwest::{Client, Url};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

mod error;
mod groups;
mod people;
mod utils;

// Re-export the main components
pub use error::MixpanelError;
pub use groups::MixpanelGroups;
pub use people::MixpanelPeople;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),
    
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Mixpanel API error: {0}")]
    Api(String),
    
    #[error("Time conversion error")]
    Time,
    
    #[error("Missing API secret for import")]
    MissingSecret,
}

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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            ip: None,
            ignore_time: None,
            time: None,
            ignore_alias: None,
            latitude: None,
            longitude: None,
        }
    }
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
    pub async fn track<S: Into<String>>(&self, event: S, properties: Option<HashMap<String, serde_json::Value>>) -> Result<()> {
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
    
    /// Import an event with a specified timestamp (for events older than 5 days)
    pub async fn import<S: Into<String>>(
        &self, 
        event: S, 
        time: u64, 
        properties: Option<HashMap<String, serde_json::Value>>
    ) -> Result<()> {
        if self.config.secret.is_none() && self.config.api_key.is_none() {
            return Err(Error::MissingSecret);
        }
        
        let mut props = properties.unwrap_or_default();
        props.insert("token".to_string(), self.token.clone().into());
        props.insert("mp_lib".to_string(), "rust".into());
        props.insert("$lib_version".to_string(), env!("CARGO_PKG_VERSION").into());
        props.insert("time".to_string(), time.into());
        
        let data = Event {
            event: event.into(),
            properties: props,
        };
        
        if self.config.debug {
            println!("Importing event to Mixpanel: {:?}", &data);
        }
        
        self.send_request("GET", "/import", &data).await
    }
    
    /// Track multiple events in a single request (batch)
    pub async fn track_batch(&self, events: Vec<Event>) -> Result<()> {
        // Process each event to ensure it has the required properties
        let events: Vec<Event> = events.into_iter().map(|event| {
            let mut props = event.properties;
            props.insert("token".to_string(), self.token.clone().into());
            props.insert("mp_lib".to_string(), "rust".into());
            props.insert("$lib_version".to_string(), env!("CARGO_PKG_VERSION").into());
            
            Event {
                event: event.event,
                properties: props,
            }
        }).collect();
        
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
    
    /// Import multiple events in a single request (batch)
    pub async fn import_batch(&self, events: Vec<Event>) -> Result<()> {
        if self.config.secret.is_none() && self.config.api_key.is_none() {
            return Err(Error::MissingSecret);
        }
        
        // Process each event to ensure it has the required properties
        let events: Vec<Event> = events.into_iter().map(|event| {
            let mut props = event.properties;
            props.insert("token".to_string(), self.token.clone().into());
            props.insert("mp_lib".to_string(), "rust".into());
            props.insert("$lib_version".to_string(), env!("CARGO_PKG_VERSION").into());
            
            // Ensure time is present for import
            if !props.contains_key("time") {
                return Err(Error::Time);
            }
            
            Ok(Event {
                event: event.event,
                properties: props,
            })
        }).collect::<Result<Vec<Event>>>()?;
        
        if self.config.debug {
            println!("Importing batch of {} events to Mixpanel", events.len());
        }
        
        // Mixpanel accepts a maximum of 50 events per request
        const MAX_BATCH_SIZE: usize = 50;
        
        for chunk in events.chunks(MAX_BATCH_SIZE) {
            self.send_request("POST", "/import", chunk).await?;
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
    
    /// Send a request to the Mixpanel API
    pub async fn send_request<T: Serialize + ?Sized>(&self, method: &str, endpoint: &str, data: &T) -> Result<()> {
        let data_json = serde_json::to_string(data)?;
        let encoded_data = BASE64.encode(data_json.as_bytes());
        
        // Build the URL with appropriate query parameters
        let mut url = Url::parse(
            &format!("{}://{}{}", self.config.protocol, self.config.host, self.config.path)
        )?;
        
        url.set_path(&format!("{}{}", url.path(), endpoint));
        
        // Add query parameters
        {
            let mut query_pairs = url.query_pairs_mut();
            
            // Add ip parameter if geolocate is enabled
            if self.config.geolocate {
                query_pairs.append_pair("ip", "1");
            } else {
                query_pairs.append_pair("ip", "0");
            }
            
            // Add verbose parameter if verbose is enabled
            if self.config.verbose {
                query_pairs.append_pair("verbose", "1");
            } else {
                query_pairs.append_pair("verbose", "0");
            }
            
            // Handle authentication for import endpoint
            if endpoint == "/import" {
                if let Some(ref api_key) = self.config.api_key {
                    query_pairs.append_pair("api_key", api_key);
                }
            }
            
            // For GET requests, include the data in the URL
            if method.to_uppercase() == "GET" {
                query_pairs.append_pair("data", &encoded_data);
            }
            
            // Add test parameter if test mode is enabled
            if self.config.test {
                query_pairs.append_pair("test", "1");
            }
        }
        
        // Create the request
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.http_client.get(url),
            "POST" => {
                let mut builder = self.http_client.post(url);
                builder = builder.header("Content-Type", "application/x-www-form-urlencoded");
                builder = builder.body(format!("data={}", encoded_data));
                builder
            },
            _ => return Err(Error::Api(format!("Unsupported HTTP method: {}", method))),
        };
        
        // Add authorization header if using API secret
        if let Some(ref secret) = self.config.secret {
            if self.config.protocol != "https" && endpoint == "/import" {
                return Err(Error::Api("Must use HTTPS if authenticating with API Secret".to_string()));
            }
            
            let auth_header = format!("Basic {}", BASE64.encode(format!("{}:", secret).as_bytes()));
            request_builder = request_builder.header("Authorization", auth_header);
        }
        
        // Send the request
        let response = request_builder.send().await?;
        
        // Check for errors
        if !response.status().is_success() {
            return Err(Error::Api(format!("HTTP error: {}", response.status())));
        }
        
        let body = response.text().await?;
        
        if self.config.verbose {
            match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(json) => {
                    if let Some(status) = json.get("status").and_then(|s| s.as_u64()) {
                        if status != 1 {
                            if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
                                return Err(Error::Api(format!("Mixpanel Server Error: {}", error)));
                            }
                        }
                    }
                },
                Err(_) => {
                    if body != "1" {
                        return Err(Error::Api(format!("Mixpanel Server Error: {}", body)));
                    }
                }
            }
        } else if body != "1" {
            return Err(Error::Api(format!("Mixpanel Server Error: {}", body)));
        }
        
        Ok(())
    }
    
    /// Get the current timestamp in seconds
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
