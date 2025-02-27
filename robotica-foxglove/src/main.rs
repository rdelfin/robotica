use clap::Parser;
use foxglove::{Schema, WebSocketServer};
use prost_reflect::DescriptorPool;
use robotica::Node;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

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
    let config_str = tokio::fs::read_to_string(args.config).await?;
    let config = toml::from_str::<config::Config>(&config_str)?;

    // Prepare the file descriptors
    let mut pool = DescriptorPool::new();
    let descriptor_pool_data = pool.encode_to_vec();
    let fd_data = load_file_descriptor_data(&config.file_desriptor_sets).await?;
    for fd in fd_data {
        let descriptor = DescriptorPool::decode(&fd[..])?;
        pool.add_file_descriptor_protos(descriptor.file_descriptor_protos().cloned())?;
    }

    let server = WebSocketServer::new();
    let node = Node::new("foxglove-server").await?;

    let channels = config
        .topics
        .into_iter()
        .map(|(topic, config)| {
            let message = pool
                .get_message_by_name(message_name_from_type_url(&config.type_url)?)
                .ok_or(anyhow::anyhow!(
                    "could not find type URL {} in file descriptors",
                    config.type_url
                ))?;
            let schema = Schema::new(&config.type_url, "protobuf", descriptor_pool_data.clone());
            Ok((
                topic.clone(),
                foxglove::ChannelBuilder::new(topic).schema(schema).build(),
            ))
        })
        .collect::<anyhow::Result<HashMap<_, _>>>()?;

    let handle = server.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
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
