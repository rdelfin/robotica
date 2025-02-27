use anyhow::Context;
use clap::Parser;
use foxglove::{Channel, Schema, WebSocketServer};
use futures::stream::{FuturesUnordered, StreamExt};
use prost::Message;
use prost_reflect::DescriptorPool;
use robotica::{Node, UntypedSubscriber};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Parser, Clone)]
struct Args {
    /// Path to the config for this node
    #[arg(short, long)]
    config: PathBuf,
}

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config_str = tokio::fs::read_to_string(args.config)
        .await
        .context("Issue reading config from file")?;
    let config = toml::from_str::<config::Config>(&config_str).context("Issue parsing config")?;

    // Prepare the file descriptors
    let mut pool = DescriptorPool::new();
    let fd_data = load_file_descriptor_data(&config.file_descriptor_sets)
        .await
        .context("error loading file descriptor data")?;
    for fd in fd_data {
        let descriptor =
            DescriptorPool::decode(&fd[..]).context("failed to parse file descriptor")?;
        pool.add_file_descriptor_protos(descriptor.file_descriptor_protos().cloned())
            .context("failed to add file descriptor to pool")?;
    }
    let descriptor_pool_data = pool.encode_to_vec();

    let server = WebSocketServer::new();
    let node = Node::new("foxglove-server").await?;

    let channels = config
        .topics
        .into_iter()
        .map(|(topic, config)| {
            let schema = Schema::new(
                message_name_from_type_url(&config.type_url)?,
                "protobuf",
                descriptor_pool_data.clone(),
            );
            Ok((
                topic.clone(),
                foxglove::ChannelBuilder::new(&topic)
                    .schema(schema)
                    .message_encoding("protobuf")
                    .build()
                    .context(format!(
                        "failed to build foxglove channel for topic {topic}"
                    ))?,
            ))
        })
        .collect::<anyhow::Result<HashMap<_, _>>>()?;
    let mut topic_futures = Vec::new();
    for (topic, channel) in channels {
        let subscriber = node.subscribe_untyped(topic).await?;
        topic_futures.push(tokio::spawn(async move {
            topic_update(channel, subscriber).await.unwrap()
        }));
    }

    let handle = server.start().await?;

    let topic_futures = FuturesUnordered::from_iter(topic_futures.into_iter());
    topic_futures
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect::<Result<(), _>>()?;

    handle.stop().await;
    Ok(())
}

async fn topic_update(
    channel: Arc<Channel>,
    mut subscriber: UntypedSubscriber,
) -> anyhow::Result<()> {
    loop {
        let data = subscriber.recv().await?;
        channel.log(&data.message.encode_to_vec());
    }
}

async fn load_file_descriptor_data(paths: &[PathBuf]) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(paths.len());
    for path in paths {
        result.push(tokio::fs::read(path).await?);
    }
    Ok(result)
}

fn message_name_from_type_url(type_url: &str) -> anyhow::Result<&str> {
    type_url
        .split('/')
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Type URL {type_url} is not a valid URL"))
}
