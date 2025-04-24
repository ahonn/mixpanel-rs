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
    let distinct_id = "user_123";
    let modifiers = Some(Modifiers::default());

    let mut properties = HashMap::new();
    properties.insert("$name".to_string(), json!("Mike"));
    properties.insert("$email".to_string(), json!("mike@example.com"));
    properties.insert("$phone".to_string(), json!("+61 123 4567 890"));
    properties.insert("age".to_string(), json!(25));
    mp.people
        .set(distinct_id, properties, modifiers.clone())
        .await?;

    let mut once_properties = HashMap::new();
    once_properties.insert("first_login_time".to_string(), json!(Mixpanel::now()));
    mp.people
        .set_once(distinct_id, once_properties, modifiers.clone())
        .await?;

    let mut number_properties = HashMap::new();
    number_properties.insert("login_count".to_string(), 1);
    number_properties.insert("total_spent".to_string(), 100);
    mp.people
        .increment(distinct_id, number_properties, modifiers.clone())
        .await?;

    let mut append_properties = HashMap::new();
    append_properties.insert("purchased_items".to_string(), json!(vec!["book", "pen"]));
    mp.people
        .append(distinct_id, append_properties, modifiers)
        .await?;

    println!("User profile updated successfully!");
    Ok(())
}
