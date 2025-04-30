use crate::book;
use crate::book::repository::diesel::entity::{delete_book_origin_data, insert_book_origins, insert_books, update_book, BookEntity, BookForm, BookOriginDataEntity, NewBookEntity, NewBookOriginDataEntity};
use crate::book::repository::diesel::{get_connection, schema, DbPool};
use book::{Book, BookRepository};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;

const MAX_BUFFER_SIZE: usize = 100;

pub struct Repository {
    pool: DbPool
}

impl Repository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

impl BookRepository for Repository {
    fn get_by_isbn<'book, I>(&self, isbn: I) -> Vec<Book>
    where
        I: Iterator<Item=&'book str>
    {
        let entities = schema::book::table
            .filter(schema::book::isbn.eq_any(isbn))
            .left_join(schema::book_origin_data::table)
            .select((
                BookEntity::as_select(),
                Option::<BookOriginDataEntity>::as_select()
            ))
            .into_boxed()
            .load(&mut get_connection(&self.pool))
            .unwrap();

        let mut books = HashMap::new();
        for (book_entity, origin) in entities {
            let entry = books.entry(book_entity.isbn.clone());
            let book = entry.or_insert_with(|| book_entity.to_domain());
            if let Some(origin) = origin {
                if let Some(val) = origin.val {
                    book.add_origin_data(origin.site, origin.property, val);
                }
            }
        }
        books.into_values().collect()
    }

    fn new_books(&self, books: &[&Book]) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);

        let new_books  = books.iter()
            .map(|book| NewBookEntity::new(book))
            .collect::<Vec<NewBookEntity>>();

        let new_book_chunks = new_books.chunks(MAX_BUFFER_SIZE);
        let new_books = new_book_chunks.into_iter()
            .flat_map(|ch| insert_books(&mut conn, ch))
            .map(|result| {
                let book = result.to_domain();
                (book.isbn.clone(), book)
            })
            .collect::<HashMap<String, Book>>();

        let new_origins = books.iter()
            .flat_map(|book| {
                let book = new_books.get(book.isbn.as_str()).unwrap();
                NewBookOriginDataEntity::new(book.id as i64, &book.origin_data)
            })
            .collect::<Vec<NewBookOriginDataEntity>>();

        let new_origin_chunks = new_origins.chunks(MAX_BUFFER_SIZE);
        new_origin_chunks.into_iter()
            .for_each(|ch| insert_book_origins(&mut conn, ch));

        new_books.into_values().collect()
    }

    fn update_books(&self, books: &[&Book]) -> Vec<Book> {
        let mut conn = get_connection(&self.pool);

        let mut updated_isbn = vec![];
        books.iter().for_each(|book| {
            let form = BookForm::new(book);
            let updated = update_book(&mut conn, &book.isbn, &form);
            if updated > 0 {
                let id = book.id as i64;
                book.origin_data.iter().for_each(|(site, _)| {
                    delete_book_origin_data(&mut conn, id, site);
                });
                let new_origins = NewBookOriginDataEntity::new(id, &book.origin_data);
                insert_book_origins(&mut conn, &new_origins);
                updated_isbn.push(book.isbn.as_str())
            }
        });

        self.get_by_isbn(updated_isbn.into_iter())
    }
}