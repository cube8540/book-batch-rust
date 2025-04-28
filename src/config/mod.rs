pub mod api;
pub mod db;
pub mod logger;

use config;
use diesel::Connection;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct Config {
    db: db::Config,
    api: api::Config,
    logger: logger::Config,
}

impl Config {
    pub fn db(&self) -> &db::Config {
        &self.db
    }

    pub fn api(&self) -> &api::Config {
        &self.api
    }

    pub fn logger(&self) -> &logger::Config {
        &self.logger
    }
}

pub fn load_config() -> Config {
    let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    let config: config::Config = config::Config::builder()
        .add_source(config::File::with_name(&format!("config/{}.json", env)))
        .build()
        .unwrap();

    config.try_deserialize()
        .unwrap()
}