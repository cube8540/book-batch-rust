use diesel::associations::HasTable;
use diesel::{PgConnection, QueryDsl, SelectableHelper};
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;

mod entity;
mod schema;

/// 출판사 도메인
#[derive(Debug)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: Vec<String>
}

impl Publisher {
    fn new(id: u64, name: String) -> Self {
        let keywords: Vec<String> = Vec::new();
        Publisher{
            id,
            name,
            keywords
        }
    }

    fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }
}

impl PartialEq<Self> for Publisher {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type Site = String;
pub type Json = String;

#[derive(Debug)]
pub struct Book {
    pub id: u64,
    pub isbn: String,
    pub publisher_id: u64,
    pub series_id: Option<u64>,
    pub title: String,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,
    pub origin_data: HashMap<Site, Json>,
}

type DbPool = r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;
type DbConnection = r2d2::PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>;

pub struct PublisherRepository {
    pool: DbPool
}

impl PublisherRepository {
    pub fn new(pool: DbPool) -> Self {
        PublisherRepository{
            pool
        }
    }

    pub fn get_publisher_all(&self) -> Vec<Publisher> {
        let mut conn = self.pool.get().expect("Failed to get db connection from pool");
        let result_set = entity::find_publisher_all(&mut conn);

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

pub struct BookRepository {
    pool: DbPool
}

impl BookRepository {
    pub fn new(pool: DbPool) -> Self {
        BookRepository{
            pool
        }
    }

    pub fn get_book_by_isbn(&self, isbn: &str) -> Vec<Book> {
        let mut conn = self.pool.get().expect("Failed to get db connection from pool");
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