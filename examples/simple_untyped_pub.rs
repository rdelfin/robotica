use robotica::{LogConfig, Node};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("simple_pub", LogConfig::new()).await?;
    let publisher = node
        .publish_untyped("test_topic", "type.googleapis.com/robotica.StringMessage")
        .await?;
    let mut i = 0;
    loop {
        let json_val = serde_json::json!({ "data": format!("Hello, World! {i}") });
        println!("Sending: {}", json_val);
        publisher.send(json_val).await?;
        i += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
