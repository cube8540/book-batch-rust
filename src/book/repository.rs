use crate::book::{entity, Book, BookRepository, Publisher, PublisherRepository};
use diesel::PgConnection;
use std::collections::HashMap;

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;
type DbConnection = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>;

pub struct DieselRepository {
    pool: DbPool
}

impl DieselRepository {
    pub fn new(pool: DbPool) -> Self {
        DieselRepository {
            pool
        }
    }
}

impl PublisherRepository for DieselRepository {
    fn get_all(&self) -> Vec<Publisher> {
        let mut conn = self.pool
            .get()
            .expect("Failed to get db connection from pool");
        let result_set = crate::book::entity::find_publisher_all(&mut conn);

        let mut map = HashMap::<u64, Publisher>::new();
        result_set.iter().for_each(|item| {
            let publisher_entity = &item.0;
            let keyword_entity = &item.1;

            let id = publisher_entity.id as u64;
            let publisher = map.entry(id)
                .or_insert_with(|| Publisher::new(id, publisher_entity.name.clone()));

            if let Some(k) = keyword_entity {
                publisher.add_keyword(k.keyword.clone())
            }
        });
        map.into_values().collect()
    }
}

pub struct DieselBookRepository {
    pool: DbPool
}

impl DieselBookRepository {
    pub fn new(pool: DbPool) -> Self {
        DieselBookRepository {
            pool
        }
    }
}

impl BookRepository for DieselBookRepository {
    fn get_by_isbn(&self, isbn: &str) -> Vec<Book> {
        let mut conn = self.pool
            .get()
            .expect("Failed to get db connection from pool");
        let result_set = entity::find_book_by_isbn(isbn, &mut conn);
        result_set.iter()
            .map(|book| {
                Book {
                    id: book.id as u64,
                    isbn: book.isbn.clone(),
                    publisher_id: book.publisher_id as u64,
                    series_id: book.series_id.map(|id| id as u64),
                    title: book.title.clone(),
                    scheduled_pub_date: book.scheduled_pub_date.clone(),
                    actual_pub_date: book.actual_pub_date.clone(),
                    origin_data: Default::default(),
                }
            })
            .collect()
    }
}
