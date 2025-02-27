use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct Config {
    topic: String,
    type_url: String,
}
