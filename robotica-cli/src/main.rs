use clap::{Parser, Subcommand};

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

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Topic { command } => topic::topic_cmd(command),
    }
}
