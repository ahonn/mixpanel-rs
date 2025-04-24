use dotenv::dotenv;
use mixpanel_rs::{Config, Event, Mixpanel};
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

    let mut events = Vec::new();

    let mut props1 = HashMap::new();
    props1.insert("user_id".to_string(), json!("user_1"));
    props1.insert("plan".to_string(), json!("premium"));
    props1.insert("time".to_string(), json!(Mixpanel::now() - 86400));
    events.push(Event {
        event: "Plan Upgraded".to_string(),
        properties: props1,
    });

    let mut props2 = HashMap::new();
    props2.insert("user_id".to_string(), json!("user_2"));
    props2.insert("amount".to_string(), json!(299.99));
    props2.insert("time".to_string(), json!(Mixpanel::now() - 43200));
    events.push(Event {
        event: "Purchase Completed".to_string(),
        properties: props2,
    });

    mp.import_batch(events.clone()).await?;
    println!("Historical events imported successfully!");

    for event in &mut events {
        event.properties.remove("time");
    }
    mp.track_batch(events).await?;
    println!("Real-time events tracked successfully!");

    Ok(())
}
