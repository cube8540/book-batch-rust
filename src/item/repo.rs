use crate::item::repo::diesel::BookPgStore;
use crate::item::repo::mongo::BookOriginDataStore;
use crate::item::{Book, BookBuilder, BookRepository};
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
    fn set_origin_data_to_builder(&self, builders: &mut [BookBuilder]) {
        let mut books_ids: Vec<i64> = vec![];
        let mut map: HashMap<i64, &mut BookBuilder> = HashMap::new();

        for builder in builders {
            let id = builder.id.unwrap() as i64;
            books_ids.push(id);
            map.insert(id, builder);
        }

        let origins = self.origin_store.find_by_book_id(&books_ids)
            .unwrap_or_else(|e| logging_with_default_vec(e));

        for origin in origins {
            let builder = map.get_mut(&origin.book_id());
            if let Some(builder) = builder {
                let (site, original) = origin.to_domain();
                builder.add_original_without_ownership(site, original);
            }
        }
    }
}

impl BookRepository for ComposeBookRepository {
    fn find_by_pub_between(&self, from: &NaiveDate, to: &NaiveDate) -> Vec<Book> {
        let mut book_builders = self.book_store
            .find_by_pub_between(from, to)
            .map(|entities| {
                entities.into_iter()
                    .map(|entity| entity.to_domain_builder())
                    .collect()
            })
            .unwrap_or_else(|e| logging_with_default_vec(e));

        if self.with_origin {
            self.set_origin_data_to_builder(&mut book_builders);
        }

        book_builders.into_iter()
            .map(|b| b.build().unwrap())
            .collect()
    }

    fn find_by_isbn(&self, isbn: &[&str]) -> Vec<Book> {
        let mut book_builders = self.book_store
            .find_by_isbn(isbn)
            .map(|entities| {
                entities.into_iter()
                    .map(|entity| entity.to_domain_builder())
                    .collect()
            })
            .unwrap_or_else(|e| logging_with_default_vec(e));

        if self.with_origin {
            self.set_origin_data_to_builder(&mut book_builders);
        }

        book_builders.into_iter()
            .map(|b| b.build().unwrap())
            .collect()
    }

    fn save_books(&self, books: &[Book]) -> Vec<Book> {
        let isbn_with_origin = books.iter()
            .map(|b| (b.isbn().to_owned(), b.originals()))
            .collect::<HashMap<_, _>>();

        let mut saved_books = self.book_store.save_books(books)
            .map(|entities| {
                entities.into_iter()
                    .map(|entity| entity.to_domain_builder())
                    .collect()
            })
            .unwrap_or_else(|e| logging_with_default_vec(e));

        if self.with_origin {
            for saved_book in saved_books.iter_mut() {
                let isbn = saved_book.isbn.as_ref().map(|isbn| isbn.clone()).unwrap();
                if let Some(original) = isbn_with_origin.get(&isbn) {
                    let saved_book_id = saved_book.id.unwrap() as i64;

                    _ = self.origin_store.new_original_data(saved_book_id, original)
                        .map_err(|e| error!("{:?}", e));

                    let saved_origins = self.origin_store
                        .find_by_book_id(&[saved_book_id])
                        .unwrap_or_else(|e| logging_with_default_vec(e));

                    for origins in saved_origins {
                        let (site, original) = origins.to_domain();
                        saved_book.add_original_without_ownership(site, original);
                    }
                }
            }
        }

        saved_books.into_iter()
            .map(|b| b.build().unwrap())
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
        let mut book_builders = self.book_store
            .find_series_unorganized(limit)
            .map(|entities| {
                entities.into_iter()
                    .map(|entity| entity.to_domain_builder())
                    .collect()
            })
            .unwrap_or_else(|e| logging_with_default_vec(e));

        if self.with_origin {
            self.set_origin_data_to_builder(&mut book_builders);
        }

        book_builders.into_iter()
            .map(|b| b.build().unwrap())
            .collect()
    }

    fn find_by_series_id(&self, series_id: u64) -> Vec<Book> {
        let mut book_builders = self.book_store
            .find_by_series_id(series_id)
            .map(|entities| {
                entities.into_iter()
                    .map(|entity| entity.to_domain_builder())
                    .collect()
            })
            .unwrap_or_else(|e| logging_with_default_vec(e));

        if self.with_origin {
            self.set_origin_data_to_builder(&mut book_builders);
        }

        book_builders.into_iter()
            .map(|b| b.build().unwrap())
            .collect()
    }
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