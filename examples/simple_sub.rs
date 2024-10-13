use chrono::{DateTime, Utc};
use robotica::{Node, Subscriber};
use robotica_types::StringMessage;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new("simple_sub").await?;
    let subscriber: Subscriber<StringMessage> = node.subscribe("test_topic").await?;
    while let Ok(msg) = subscriber.recv().await {
        let system_time: SystemTime = msg
            .header
            .message_timestamp
            .expect("header timestamp not set")
            .try_into()?;
        let date_time: DateTime<Utc> = system_time.into();
        println!(
            "Received: {:?} (sent at {})",
            msg.message,
            date_time.to_rfc3339()
        );
    }
    Ok(())
}
