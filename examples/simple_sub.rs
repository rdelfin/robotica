use robotica::Node;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let node = Node::new("simple_sub".to_string()).await?;
    let subscriber = node.subscribe("test_topic".to_string()).await?;
    while let Ok(value) = subscriber.recv().await {
        println!("Received: \"{value}\"");
    }
    Ok(())
}
