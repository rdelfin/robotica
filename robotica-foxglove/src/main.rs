use std::time::Duration;

use foxglove::WebSocketServer;
use robotica::Node;

mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = WebSocketServer::new();
    let node = Node::new("foxglove-server").await?;
    let handle = server.start().await?;

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
