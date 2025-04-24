use dotenv::dotenv;
use mixpanel_rs::{Config, Mixpanel};
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

    mp.track("Simple Event", None).await?;

    let mut properties = HashMap::new();
    properties.insert("user_type".to_string(), json!("new"));
    properties.insert("source".to_string(), json!("register"));
    mp.track("User Registered", Some(properties)).await?;

    mp.alias("old_id_123", "new_id_456").await?;

    println!("Events tracked successfully!");
    Ok(())
}
