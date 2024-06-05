use robotica::Node;
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new("simple_pub".to_string()).await?;
    let publisher = node.publish("test_topic".to_string()).await?;
    let mut i = 0;
    loop {
        let msg = format!("Hello, World! {i}");
        println!("Sending: \"{msg}\"");
        publisher.send(msg).await?;
        i += 1;
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}
