use dotenv::dotenv;
use mixpanel_rs::{Config, Mixpanel, Modifiers};
use serde_json::json;
use std::{collections::HashMap, env};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let project_token = env::var("MIXPANEL_PROJECT_TOKEN")
        .expect("MIXPANEL_PROJECT_TOKEN must be set in .env file");
    let api_secret =
        env::var("MIXPANEL_API_SECRET").expect("MIXPANEL_API_SECRET must be set in .env file");

    let config = Config {
        secret: Some(api_secret),
        debug: true,
        ..Default::default()
    };
    let mp = Mixpanel::init(&project_token, Some(config));

    let group_key = "company";
    let group_id = "company_123";
    let modifiers = Some(Modifiers::default());

    let mut properties = HashMap::new();
    properties.insert("name".to_string(), json!("Example Tech"));
    properties.insert("industry".to_string(), json!("technology"));
    properties.insert("employee_count".to_string(), json!(100));
    mp.groups
        .set(group_key, group_id, properties, modifiers.clone())
        .await?;

    let mut once_properties = HashMap::new();
    once_properties.insert("founded_time".to_string(), json!(Mixpanel::now()));
    mp.groups
        .set_once(group_key, group_id, once_properties, modifiers.clone())
        .await?;

    mp.groups
        .delete_group(group_key, group_id, modifiers.clone())
        .await?;

    let mut remove_properties = HashMap::new();
    remove_properties.insert("employee_count".to_string(), json!(null));
    mp.groups
        .remove(group_key, group_id, remove_properties, modifiers.clone())
        .await?;

    let mut event_properties = HashMap::new();
    event_properties.insert("$group_key".to_string(), json!(group_key));
    event_properties.insert("$group_id".to_string(), json!(group_id));
    event_properties.insert("action".to_string(), json!("upgrade"));
    mp.track("Company Action", Some(event_properties)).await?;

    println!("Group operations completed successfully!");
    Ok(())
}
