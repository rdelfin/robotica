use super::TopicCommands;

pub fn topic_cmd(command: TopicCommands) {
    match command {
        TopicCommands::List => topic_list(),
        TopicCommands::Sub { topic_name } => topic_sub(topic_name),
    }
}

fn topic_list() {
    unimplemented!();
}

fn topic_sub(name: String) {
    unimplemented!();
}
