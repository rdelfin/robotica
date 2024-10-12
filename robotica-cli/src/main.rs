use clap::{Parser, Subcommand};
use robotica::Node;

mod topic;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Topic {
        #[command(subcommand)]
        command: TopicCommands,
    },
}

/// A collection of all commands relating to listing, printing, and managing topics.
#[derive(Subcommand, Debug)]
enum TopicCommands {
    /// Prints out all messages published on a given topic name. Short for subscribe
    Sub {
        /// Name of the topic subscribed to
        topic_name: String,
    },
    /// Lists out all topics currently active and publishing
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let node = Node::new("cli".into()).await?;
    match args.command {
        Commands::Topic { command } => topic::topic_cmd(node, command).await,
    }
}
