use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub topics: HashMap<String, TopicConfig>,
    pub file_descriptor_sets: Vec<PathBuf>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicConfig {
    pub type_url: String,
}
