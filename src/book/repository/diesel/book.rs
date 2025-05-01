use crate::book;
use crate::book::repository::diesel::{entity, get_connection, schema, DbPool};
use crate::book::repository::BookRepository;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;

const MAX_BUFFER_SIZE: usize = 100;

pub struct Repository {
    pool: DbPool
}

pub fn new(pool: DbPool) -> Repository {
    Repository { pool }
}

impl BookRepository for Repository {
    fn find_by_isbn<'book, I>(&self, isbn: I) -> Vec<book::Book>
    where
        I: Iterator<Item=&'book str>
    {
        let entities: Vec<entity::Book> = schema::book::table
            .filter(schema::book::isbn.eq_any(isbn))
            .select(entity::Book::as_select())
            .into_boxed()
            .load(&mut get_connection(&self.pool))
            .unwrap();

        entities.iter()
            .map(|e| e.to_domain())
            .collect()
    }

    fn find_origin_by_id<I>(&self, id: I) -> HashMap<u64, HashMap<book::Site, book::Original>>
    where
        I: Iterator<Item=u64>
    {
        let mut result: HashMap<u64, HashMap<book::Site, book::Original>> = HashMap::new();

        let id = id.map(|id| id.clone() as i64);
        let origins: Vec<entity::BookOriginData> = schema::book_origin_data::table
            .filter(schema::book_origin_data::book_id.eq_any(id))
            .select(entity::BookOriginData::as_select())
            .load(&mut get_connection(&self.pool))
            .unwrap();

        origins.into_iter().for_each(|entity| {
            if let Some(v) = entity.val {
                let id = entity.book_id as u64;
                result.entry(id)
                    .or_insert_with(|| HashMap::new()).entry(entity.site)
                    .or_insert_with(|| HashMap::new())
                    .insert(entity.property, v);
            }
        });
        result
    }

    fn new_books<'book, I>(&self, books: I, with_origin: bool) -> Vec<book::Book>
    where
        I: IntoIterator<Item=&'book book::Book>
    {
        let mut mapping_books: HashMap<String, &book::Book> = HashMap::new();
        let mut new_books: Vec<entity::NewBook> = vec![];

        for book in books {
            mapping_books.insert(book.isbn.clone(), book);
            new_books.push(entity::NewBook::new(book));
        }

        let mut connection = get_connection(&self.pool);
        let registered_books: Vec<book::Book> = new_books.chunks(MAX_BUFFER_SIZE).into_iter()
            .flat_map(|books| {
                diesel::insert_into(schema::book::table)
                    .values(books)
                    .get_results::<entity::Book>(&mut connection)
                    .expect("Error inserting new books.")
            })
            .map(|result| result.to_domain())
            .collect();

        if with_origin {
            let new_origins = registered_books
                .iter()
                .filter_map(|b| mapping_books.get(&b.isbn).map(|b| (b.id, &b.origin_data)));
            self.new_origin_data(new_origins);
        }
        registered_books

    }

    fn new_origin_data<'book, I>(&self, origins: I) -> usize
    where
        I: IntoIterator<Item=(u64, &'book HashMap<book::Site, book::Original>)>
    {
        let new_origins: Vec<entity::NewBookOriginDataEntity> = origins
            .into_iter()
            .flat_map(|(id, original)| {
                original.iter()
                    .flat_map(move |(key, val)| {
                        entity::NewBookOriginDataEntity::new(id as i64, key, val)
                    })
            })
            .collect();

        let mut connection = get_connection(&self.pool);
        new_origins.chunks(MAX_BUFFER_SIZE).into_iter()
            .flat_map(|origins| {
                diesel::insert_into(schema::book_origin_data::table)
                    .values(origins)
                    .execute(&mut connection)
            })
            .sum()
    }

    fn update_books<'book, I>(&self, books: I, with_origin: bool) -> usize
    where
        I: IntoIterator<Item=&'book book::Book>
    {
        let mut connection = get_connection(&self.pool);
        books.into_iter()
            .map(|book| {
                let form = entity::BookForm::new(book);
                let mut count = diesel::update(schema::book::table)
                    .filter(schema::book::id.eq(book.id as i64))
                    .set(form)
                    .execute(&mut connection)
                    .unwrap();
                if count > 0 && with_origin {
                    book.origin_data.iter().for_each(|(site, _)| {
                        self.delete_origin_data(book.id.clone(), site);
                    });
                    count += self.new_origin_data([(book.id, &book.origin_data)]);
                }
                count
            })
            .sum()
    }

    fn delete_origin_data(&self, id: u64, site: &book::Site) -> usize {
        diesel::delete(schema::book_origin_data::dsl::book_origin_data
            .filter(
                schema::book_origin_data::book_id.eq(id as i64)
                    .and(schema::book_origin_data::site.eq(site))
            ))
            .into_boxed()
            .execute(&mut get_connection(&self.pool))
            .unwrap()
    }
}