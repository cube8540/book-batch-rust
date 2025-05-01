use crate::book;
use crate::book::repository::diesel::{entity, get_connection, schema, sql_debugging, DbPool};
use crate::book::repository::PublisherRepository;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;

pub struct Repository {
    pool: DbPool
}

pub fn new(pool: DbPool) -> Repository {
    Repository { pool }
}

impl PublisherRepository for Repository {
    fn get_all(&self) -> Vec<book::Publisher> {
        let entities = sql_debugging(schema::publisher::table
            .left_join(schema::publisher_keyword::table)
            .select((
                entity::Publisher::as_select(),
                Option::<entity::PublisherKeyword>::as_select()
            )))
            .into_boxed()
            .load(&mut get_connection(&self.pool))
            .unwrap();

        let mut map = HashMap::<u64, book::Publisher>::new();
        entities.iter().for_each(|(publisher, keyword)| {
            let id = publisher.id as u64;
            let publisher = map.entry(id)
                .or_insert_with(|| book::Publisher::new(id, publisher.name.clone()));

            if let Some(k) = keyword {
                publisher.add_keyword(k.site.clone(), k.keyword.clone());
            }
        });
        map.into_values().collect()
    }
}