use robotica::Node;
use robotica_types::DESCRIPTOR_SET_BYTES;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new("simple_pub".to_string()).await?;
    let publisher = node
        .publish_untyped(
            "test_topic".to_string(),
            "type.googleapis.com/robotica.StringMessage".to_string(),
            &[DESCRIPTOR_SET_BYTES],
        )
        .await?;
    let mut i = 0;
    loop {
        let json_val = serde_json::json!({ "data": format!("Hello, World! {i}") });
        println!("Sending: {}", json_val.to_string());
        publisher.send(json_val).await?;
        i += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
