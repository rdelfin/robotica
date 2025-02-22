use super::NodeCommands;
use robotica::Node;

#[allow(clippy::module_name_repetitions)]
pub async fn node_cmd(node: Node, command: NodeCommands) -> anyhow::Result<()> {
    match command {
        NodeCommands::List => node_list(node).await,
    }
}

async fn node_list(node: Node) -> anyhow::Result<()> {
    let nodes = node.list_nodes().await?;
    for node in nodes {
        println!("{node}");
    }
    Ok(())
}
