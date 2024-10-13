use robotica::{LogConfig, Node};
use robotica_types::StringMessage;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new_with_logging("simple_pub", LogConfig::new()).await?;
    let publisher = node.publish("test_topic").await?;
    let mut i = 0;
    loop {
        let data = format!("Hello, World! {i}");
        let msg = StringMessage { data };
        println!("Sending: \"{msg:?}\"");
        publisher.send(&msg).await?;
        i += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
