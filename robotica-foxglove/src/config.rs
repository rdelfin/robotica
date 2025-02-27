use serde::Deserialize;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub topics: HashMap<String, TopicConfig>,
    pub file_descriptor_sets: Vec<PathBuf>,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicConfig {
    pub type_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_address")]
    pub address: IpAddr,
}

#[rustfmt::skip] fn default_port() -> u16 { 8765 }
#[rustfmt::skip] fn default_address() -> IpAddr { "127.0.0.1".parse().unwrap() }
