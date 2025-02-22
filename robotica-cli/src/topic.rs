use super::TopicCommands;
use robotica::Node;
use std::collections::BTreeSet;
use std::time::Duration;

#[allow(clippy::module_name_repetitions)]
pub async fn topic_cmd(node: Node, command: TopicCommands) -> anyhow::Result<()> {
    match command {
        TopicCommands::List => topic_list(node).await,
        TopicCommands::Sub { topic_name } => topic_sub(node, topic_name).await,
        TopicCommands::Pub {
            topic_name,
            topic_type,
            data,
            frequency_hz,
            repetitions,
        } => {
            topic_pub(
                node,
                topic_name,
                topic_type,
                data,
                Duration::from_secs_f32(1. / frequency_hz),
                repetitions,
            )
            .await
        }
    }
}

#[allow(clippy::unused_async)]
async fn topic_list(node: Node) -> anyhow::Result<()> {
    let mut topics = BTreeSet::new();
    let nodes = node.list_nodes().await?;
    for node_name in nodes {
        for topic in node.list_nodes_subscribers(&node_name).await? {
            topics.insert(topic.name);
        }
        for topic in node.list_nodes_publishers(&node_name).await? {
            topics.insert(topic.name);
        }
    }

    for topic in topics {
        println!("{topic}");
    }

    Ok(())
}

async fn topic_sub(node: Node, name: String) -> anyhow::Result<()> {
    let mut subscriber = node.subscribe_untyped(name).await?;
    while let Ok(msg) = subscriber.recv().await {
        println!("Got: {}", msg.message);
    }
    Ok(())
}

async fn topic_pub(
    node: Node,
    topic: String,
    type_url: String,
    data: String,
    period: Duration,
    repetitions: Option<usize>,
) -> anyhow::Result<()> {
    let publisher = node.publish_untyped(topic, type_url).await?;
    let json_value: serde_json::Value = serde_json::from_str(&data)?;
    let repetition_str = if let Some(repetitions) = &repetitions {
        format!(" {repetitions} time(s)")
    } else {
        String::new()
    };
    println!(
        "Sending {json_value} every {}s{repetition_str}...",
        period.as_secs()
    );

    let mut idx: usize = 0;
    loop {
        publisher.send(json_value.clone()).await?;
        if let Some(repetitions) = &repetitions {
            idx += 1;
            if idx >= *repetitions {
                break;
            }
        }
        tokio::time::sleep(period).await;
    }
    Ok(())
}
