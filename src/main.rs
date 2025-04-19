use crate::book::entity::{Publisher, PublisherKeyword};
use crate::book::schema::publisher::dsl::publisher;
use diesel::{BelongingToDsl, Connection, IntoSql, PgConnection, QueryDsl, RunQueryDsl, SelectableHelper};

mod book;
mod config;

fn main() {
    let config = config::load_config().unwrap();

    let db = config.db();
    let database_url = format!("postgres://{}:{}@{}:{}/{}", db.username(), db.password(), db.host(), db.port(), db.dbname());
    println!("{database_url}");

    let mut conn = PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    let publisher_sql = publisher
        .select(Publisher::as_select());
    let publisher_sql_debug = diesel::debug_query::<diesel::pg::Pg, _>(&publisher_sql).to_string();

    let publishers = publisher_sql
        .get_results(&mut conn)
        .expect("Error loading publisher");

    let publisher_keyword_sql = PublisherKeyword::belonging_to(&publishers)
        .select(PublisherKeyword::as_select());
    let keyword_sql_debug = diesel::debug_query::<diesel::pg::Pg, _>(&publisher_keyword_sql).to_string();
    let publisher_keywords = publisher_keyword_sql
        .load(&mut conn)
        .expect("Error loading publisher_keyword");

    println!("{:?}", publisher_sql_debug);
    println!("{:?}", keyword_sql_debug);
    println!("{publishers:#?} \n ${publisher_keywords:#?}")
}