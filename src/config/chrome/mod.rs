use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    server_url: String
}

impl Config {
    pub fn server_url(&self) -> &str {
        &self.server_url
    }
}