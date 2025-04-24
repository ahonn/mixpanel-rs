use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

/// Convert a timestamp to Unix epoch seconds
#[allow(dead_code)]
pub fn ensure_timestamp(time: Option<u64>) -> Option<u64> {
    time.map(|t| {
        if t > 9999999999 {
            t / 1000 // Convert milliseconds to seconds
        } else {
            t
        }
    })
}

/// Get current Unix timestamp in seconds
#[allow(dead_code)]
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

/// Merge modifiers into a data map
pub fn merge_modifiers(mut data: Value, modifiers: Option<crate::Modifiers>) -> Value {
    if let Some(modifiers) = modifiers {
        if let Some(ip) = modifiers.ip {
            data.as_object_mut()
                .unwrap()
                .insert("$ip".to_string(), ip.into());
        }
        if let Some(ignore_time) = modifiers.ignore_time {
            data.as_object_mut()
                .unwrap()
                .insert("$ignore_time".to_string(), ignore_time.into());
        }
        if let Some(time) = modifiers.time {
            data.as_object_mut()
                .unwrap()
                .insert("$time".to_string(), time.into());
        }
        if let Some(ignore_alias) = modifiers.ignore_alias {
            data.as_object_mut()
                .unwrap()
                .insert("$ignore_alias".to_string(), ignore_alias.into());
        }
        if let (Some(lat), Some(lon)) = (modifiers.latitude, modifiers.longitude) {
            data.as_object_mut()
                .unwrap()
                .insert("$latitude".to_string(), lat.into());
            data.as_object_mut()
                .unwrap()
                .insert("$longitude".to_string(), lon.into());
        }
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Modifiers;

    #[test]
    fn test_ensure_timestamp() {
        assert_eq!(ensure_timestamp(Some(1234567890)), Some(1234567890));
        assert_eq!(ensure_timestamp(Some(1234567890123)), Some(1234567890));
        assert_eq!(ensure_timestamp(None), None);
    }

    #[test]
    fn test_merge_modifiers() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            ip: Some("1.2.3.4".to_string()),
            ignore_time: Some(true),
            time: Some(1234567890),
            ignore_alias: Some(true),
            latitude: Some(40.7127753),
            longitude: Some(-74.0059728),
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("$ip").unwrap().as_str().unwrap(), "1.2.3.4");
        assert!(obj.get("$ignore_time").unwrap().as_bool().unwrap());
        assert_eq!(obj.get("$time").unwrap().as_u64().unwrap(), 1234567890);
        assert!(obj.get("$ignore_alias").unwrap().as_bool().unwrap());
        assert_eq!(obj.get("$latitude").unwrap().as_f64().unwrap(), 40.7127753);
        assert_eq!(
            obj.get("$longitude").unwrap().as_f64().unwrap(),
            -74.0059728
        );
    }

    #[test]
    fn test_merge_modifiers_ip_only() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            ip: Some("1.2.3.4".to_string()),
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert_eq!(obj.get("$ip").unwrap().as_str().unwrap(), "1.2.3.4");
        assert!(obj.get("$ignore_time").is_none());
        assert!(obj.get("$time").is_none());
        assert!(obj.get("$ignore_alias").is_none());
        assert!(obj.get("$latitude").is_none());
        assert!(obj.get("$longitude").is_none());
    }

    #[test]
    fn test_merge_modifiers_ignore_time_only() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            ignore_time: Some(true),
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert!(obj.get("$ip").is_none());
        assert!(obj.get("$ignore_time").unwrap().as_bool().unwrap());
        assert!(obj.get("$time").is_none());
        assert!(obj.get("$ignore_alias").is_none());
        assert!(obj.get("$latitude").is_none());
        assert!(obj.get("$longitude").is_none());
    }

    #[test]
    fn test_merge_modifiers_time_only() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            time: Some(1234567890),
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert!(obj.get("$ip").is_none());
        assert!(obj.get("$ignore_time").is_none());
        assert_eq!(obj.get("$time").unwrap().as_u64().unwrap(), 1234567890);
        assert!(obj.get("$ignore_alias").is_none());
        assert!(obj.get("$latitude").is_none());
        assert!(obj.get("$longitude").is_none());
    }

    #[test]
    fn test_merge_modifiers_ignore_alias_only() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            ignore_alias: Some(true),
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert!(obj.get("$ip").is_none());
        assert!(obj.get("$ignore_time").is_none());
        assert!(obj.get("$time").is_none());
        assert!(obj.get("$ignore_alias").unwrap().as_bool().unwrap());
        assert!(obj.get("$latitude").is_none());
        assert!(obj.get("$longitude").is_none());
    }

    #[test]
    fn test_merge_modifiers_geo_only() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            latitude: Some(40.7127753),
            longitude: Some(-74.0059728),
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert!(obj.get("$ip").is_none());
        assert!(obj.get("$ignore_time").is_none());
        assert!(obj.get("$time").is_none());
        assert!(obj.get("$ignore_alias").is_none());
        assert_eq!(obj.get("$latitude").unwrap().as_f64().unwrap(), 40.7127753);
        assert_eq!(
            obj.get("$longitude").unwrap().as_f64().unwrap(),
            -74.0059728
        );
    }

    #[test]
    fn test_merge_modifiers_latitude_only_should_not_add_geo() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            latitude: Some(40.7127753),
            longitude: None,
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert!(obj.get("$latitude").is_none());
        assert!(obj.get("$longitude").is_none());
    }

    #[test]
    fn test_merge_modifiers_longitude_only_should_not_add_geo() {
        let data = serde_json::json!({
            "test": "value"
        });

        let modifiers = Modifiers {
            latitude: None,
            longitude: Some(-74.0059728),
            ..Default::default()
        };

        let result = merge_modifiers(data, Some(modifiers));
        let obj = result.as_object().unwrap();

        assert!(obj.get("$latitude").is_none());
        assert!(obj.get("$longitude").is_none());
    }

    #[test]
    fn test_merge_modifiers_none() {
        let data = serde_json::json!({
            "test": "value"
        });

        let result = merge_modifiers(data.clone(), None);

        // Should return data unchanged
        assert_eq!(result, data);
    }
}

