use super::TopicCommands;
use robotica::Node;

pub async fn topic_cmd(node: Node, command: TopicCommands) -> anyhow::Result<()> {
    match command {
        TopicCommands::List => topic_list().await,
        TopicCommands::Sub { topic_name } => topic_sub(node, topic_name).await,
    }
}

async fn topic_list() -> anyhow::Result<()> {
    unimplemented!();
}

async fn topic_sub(node: Node, name: String) -> anyhow::Result<()> {
    let mut subscriber = node.subscribe_untyped(name).await?;
    while let Ok(msg) = subscriber.recv().await {
        println!("Got: {}", msg.message);
    }
    Ok(())
}
