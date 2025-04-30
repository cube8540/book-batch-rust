use crate::book::repository::diesel::entity::{PublisherEntity, PublisherKeywordEntity};
use crate::book::repository::diesel::{get_connection, schema, DbPool};
use crate::book::repository::PublisherRepository;
use crate::book::Publisher;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;

pub struct Repository {
    pool: DbPool
}

impl Repository {
    pub fn new (pool: DbPool) -> Self {
        Self { pool }
    }
}

impl PublisherRepository for Repository {
    fn get_all(&self) -> Vec<Publisher> {
        let entities = schema::publisher::table
            .left_join(schema::publisher_keyword::table)
            .select((
                PublisherEntity::as_select(),
                Option::<PublisherKeywordEntity>::as_select()
            ))
            .into_boxed()
            .load(&mut get_connection(&self.pool))
            .unwrap();

        let mut map = HashMap::<u64, Publisher>::new();
        entities.iter().for_each(|(publisher, keyword)| {
            let id = publisher.id as u64;
            let publisher = map.entry(id)
                .or_insert_with(|| Publisher::new(id, publisher.name.clone()));

            if let Some(k) = keyword {
                publisher.add_keyword(k.site.clone(), k.keyword.clone());
            }
        });
        map.into_values().collect()
    }
}