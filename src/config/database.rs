use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Database {
    host: String,
    port: i32,
    username: String,
    password: String,
    dbname: String,
}

impl Database {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> i32 {
        self.port
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn dbname(&self) -> &str {
        &self.dbname
    }
}