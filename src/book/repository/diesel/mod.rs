use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;

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