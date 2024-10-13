use clap::{Parser, Subcommand};
use robotica::{log::LevelFilter, LogConfig, Node};
use std::path::PathBuf;

mod topic;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(long, short)]
    file_descriptors_paths: Vec<PathBuf>,
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
    Pub {
        /// Name of the topic subscribed to
        topic_name: String,
        /// The type of the message we're sending
        topic_type: String,
        /// The JSON data to send
        data: String,
        /// Frequency at which to send the data
        #[arg(short, long, default_value_t = 1.)]
        frequency_hz: f32,
        /// How many messages to send. If not specified, it will run nonstop
        #[arg(short, long)]
        repetitions: Option<usize>,
    },
    /// Lists out all topics currently active and publishing
    List,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    let mut node =
        Node::new_with_logging("cli", LogConfig::new().robotica_level(LevelFilter::Warn)).await?;
    let file_descriptors = args
        .file_descriptors_paths
        .into_iter()
        .map(std::fs::read)
        .collect::<Result<Vec<_>, _>>()?;
    if !file_descriptors.is_empty() {
        for file_descriptor in file_descriptors {
            node.add_file_descriptors(&file_descriptor);
        }
    }

    match args.command {
        Commands::Topic { command } => topic::topic_cmd(node, command).await,
    }
}
