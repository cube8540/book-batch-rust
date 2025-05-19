use diesel::pg::Pg;
use diesel::r2d2::ConnectionManager;
use diesel::{debug_query, PgConnection};
use tracing::{debug, enabled};

pub mod book;
pub mod book_origin_filter;
pub mod publisher;
mod entity;
mod schema;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

fn get_connection(pool: &DbPool) -> DbConnection {
    pool.get().expect("Failed to get db connection from pool")
}

pub fn sql_debugging<T>(sql: T) -> T
where T: diesel::query_builder::QueryFragment<Pg>,
{
    if enabled!(tracing::Level::DEBUG) {
        let debug_str = debug_query::<Pg, _>(&sql).to_string();
        debug!("SQL: {}", debug_str);
    }
    sql
}