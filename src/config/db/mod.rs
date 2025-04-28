use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    host: String,
    port: i32,
    username: String,
    password: String,
    dbname: String,
}

pub fn connect_to_database(db: &Config) -> Pool<ConnectionManager<PgConnection>> {
    let database_url = format!("postgres://{}:{}@{}:{}/{}", &db.username, &db.password, &db.host, &db.port, &db.dbname);
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}