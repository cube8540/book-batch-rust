mod database;
mod api;

use crate::config::api::API;
use crate::config::database::Database;
use config;
use diesel::{Connection, PgConnection};
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    db: Database,
    api: API,
}

impl AppConfig {
    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn api(&self) -> &API {
        &self.api
    }
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    let config = config::Config::builder()
        .add_source(config::File::with_name(&format!("config/{}.json", env)))
        .build()?;

    config.try_deserialize()
}

pub fn connect_to_database(db: &Database) -> PgConnection {
    let database_url = format!("postgres://{}:{}@{}:{}/{}", db.username(), db.password(), db.host(), db.port(), db.dbname());

    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}