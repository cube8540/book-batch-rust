use crate::book::Original;
use crate::item::repo::diesel::{BookEntity, BookPgStore, PublisherPgStore};
use crate::item::repo::mongo::BookOriginDataStore;
use crate::item::{Book, BookBuilder, BookRepository, Publisher, PublisherRepository, Site};
use chrono::NaiveDate;
use ::diesel::r2d2::ConnectionManager;
use ::diesel::PgConnection;
use mongodb::sync;
use r2d2::Pool;
use std::collections::HashMap;
use std::fmt::Debug;
use sync::Client;
use tracing::error;

mod diesel;
mod mongo;

pub struct ComposeBookRepository {
    book_store: BookPgStore,
    origin_store: BookOriginDataStore,

    with_origin: bool,
}

impl ComposeBookRepository {
    pub fn without_origin(db_pool: Pool<ConnectionManager<PgConnection>>, mongo_client: Client) -> Self {
        Self {
            book_store: BookPgStore::new(db_pool),
            origin_store: BookOriginDataStore::new(mongo_client),
            with_origin: false,
        }
    }

    pub fn with_origin(db_pool: Pool<ConnectionManager<PgConnection>>, mongo_client: Client) -> Self {
        Self {
            book_store: BookPgStore::new(db_pool),
            origin_store: BookOriginDataStore::new(mongo_client),
            with_origin: true,
        }
    }
}

impl ComposeBookRepository {
    fn load_original_data(&self, entities: &[BookEntity]) -> HashMap<i64, (Site, Original)> {
        let book_ids = entities.iter()
            .map(|e| e.id)
            .collect::<Vec<_>>();

        let originals = self.origin_store.find_by_book_id(&book_ids)
            .unwrap_or_else(|e| logging_with_default_vec(e));

        originals.into_iter()
            .map(|origin| {
                let book_id = origin.book_id();
                let (site, original) = origin.to_domain();
                (book_id, (site, original))
            })
            .collect()
    }
}

impl BookRepository for ComposeBookRepository {
    fn find_by_pub_between(&self, from: &NaiveDate, to: &NaiveDate) -> Vec<Book> {
        let book_entities = self.book_store
            .find_by_pub_between(from, to)
            .unwrap_or_else(|e| logging_with_default_vec(e));

        let mut originals = match self.with_origin {
            true => self.load_original_data(&book_entities),
            false => HashMap::new(),
        };

        book_entities.into_iter()
            .map(|entity| compose_entity_with_original(entity, &mut originals))
            .collect()
    }

    fn find_by_isbn(&self, isbn: &[&str]) -> Vec<Book> {
        let book_entities = self.book_store
            .find_by_isbn(isbn)
            .unwrap_or_else(|e| logging_with_default_vec(e));

        let mut originals = match self.with_origin {
            true => self.load_original_data(&book_entities),
            false => HashMap::new(),
        };

        book_entities.into_iter()
            .map(|entity| compose_entity_with_original(entity, &mut originals))
            .collect()
    }

    fn save_books(&self, books: &[Book]) -> Vec<Book> {
        let mut isbn_with_origin = books.iter()
            .map(|b| (b.isbn().to_owned(), b.originals()))
            .collect::<HashMap<_, _>>();

        let saved_book_entities = self.book_store.save_books(books)
            .unwrap_or_else(|e| logging_with_default_vec(e));

        if saved_book_entities.len() == 0 {
            return vec![];
        }

        if self.with_origin {
            saved_book_entities.iter()
                .filter_map(|e| {
                    isbn_with_origin.get(&e.isbn).map(|o| (e.id, o))
                })
                .for_each(|(id, original)| {
                    _ = self.origin_store.new_original_data(id, original)
                        .unwrap_or_else(|e| logging_with_default_usize(e));
                });
        }

        saved_book_entities.into_iter()
            .map(|e| {
                let mut builder = e.to_domain_builder();
                if let Some(originals) = isbn_with_origin.remove(&e.isbn) {
                    for (site, original) in originals.into_iter() {
                        builder = builder.add_original(site.clone(), original.clone());
                    }
                }
                builder.build().unwrap()
            })
            .collect()
    }

    fn update_book(&self, book: &Book) -> usize {
        let mut updated_count = self.book_store.update_book(book)
            .unwrap_or_else(|e| logging_with_default_usize(e));

        if self.with_origin {
            let book_id = book.id as i64;
            for (site, _) in book.originals.iter() {
                _ = self.origin_store.delete_site(book_id, site)
                    .unwrap_or_else(|e| logging_with_default_usize(e));
            }
            updated_count += self.origin_store.new_original_data(book_id, book.originals())
                .unwrap_or_else(|e| logging_with_default_usize(e));
        }

        updated_count
    }

    fn find_series_unorganized(&self, limit: usize) -> Vec<Book> {
        let book_entities = self.book_store
            .find_series_unorganized(limit)
            .unwrap_or_else(|e| logging_with_default_vec(e));
        
        let mut originals = match self.with_origin {
            true => self.load_original_data(&book_entities),
            false => HashMap::new(),
        };
        
        book_entities.into_iter()
            .map(|entity| compose_entity_with_original(entity, &mut originals))
            .collect()
    }

    fn find_by_series_id(&self, series_id: u64) -> Vec<Book> {
        let book_entities = self.book_store
            .find_by_series_id(series_id)
            .unwrap_or_else(|e| logging_with_default_vec(e));
        
        let mut originals = match self.with_origin {
            true => self.load_original_data(&book_entities),
            false => HashMap::new(),
        };
        
        book_entities.into_iter()
            .map(|entity| compose_entity_with_original(entity, &mut originals))
            .collect()
    }
}

pub struct DieselPublisherRepository {
    store: PublisherPgStore
}

impl DieselPublisherRepository {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self {
            store: PublisherPgStore::new(pool),
        }
    }
}

impl PublisherRepository for DieselPublisherRepository {

    fn get_all(&self) -> Vec<Publisher> {
        let publisher_with_keywords = self.store.find_all()
            .unwrap_or_else(|e| logging_with_default_vec(e));
        if publisher_with_keywords.len() == 0 {
            return vec![];
        }

        let mut publisher_map: HashMap<i64, Publisher> = HashMap::new();
        for (publisher, keyword) in publisher_with_keywords.iter() {
            let publisher = publisher_map.entry(publisher.id)
                .or_insert_with(|| {
                    Publisher::without_keywords(publisher.id as u64, publisher.name.clone())
                });

            if let Some(keyword) = keyword {
                publisher.add_keyword(Site::from_str(keyword.site.as_str()).unwrap(), keyword.keyword.clone());
            }
        }

        publisher_map.into_values().collect()
    }
}

fn compose_entity_with_original(book_entity: BookEntity, originals: &mut HashMap<i64, (Site, Original)>) -> Book {
    let mut builder = book_entity.to_domain_builder();
    if let Some((site, original)) = originals.remove(&book_entity.id) {
        builder = builder.add_original(site, original);
    }
    builder.build().unwrap()
}

fn logging_with_default_usize<E>(e: E) -> usize
where
    E: Debug
{
    error!("{:?}", e);
    0
}

fn logging_with_default_vec<E, R>(e: E) -> Vec<R>
where
    E: Debug
{
    error!("{:?}", e);
    vec![]
}