mod database;
mod api;
pub mod log;

use crate::config::api::API;
use crate::config::database::Database;
use config;
use diesel::{Connection, PgConnection};
use serde::Deserialize;
use std::env;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    db: Database,
    api: API,
    logger: log::Config,
}

impl AppConfig {
    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn api(&self) -> &API {
        &self.api
    }

    pub fn logger(&self) -> &log::Config {
        &self.logger
    }
}

pub fn load_config() -> Result<AppConfig, config::ConfigError> {
    let env = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
    let config = config::Config::builder()
        .add_source(config::File::with_name(&format!("config/{}.json", env)))
        .build()?;

    config.try_deserialize()
}

pub fn connect_to_database(db: &Database) -> Pool<ConnectionManager<PgConnection>> {
    let database_url = format!("postgres://{}:{}@{}:{}/{}", db.username(), db.password(), db.host(), db.port(), db.dbname());
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}