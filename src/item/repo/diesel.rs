use crate::item::Book;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

mod schema;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    ConnectError(String),

    SqlExecuteError(String)
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::books::book)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BookEntity {
    pub id: i64,
    pub isbn: String,
    pub publisher_id: i64,
    pub series_id: Option<i64>,
    pub title: String,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,

    pub registered_at : chrono::NaiveDateTime,
    pub modified_at: Option<chrono::NaiveDateTime>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::books::book)]
pub struct NewBook<'a> {
    pub isbn: &'a str,
    pub publisher_id: i64,
    pub series_id: Option<i64>,
    pub title: &'a str,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,
    pub registered_at : chrono::NaiveDateTime
}

impl <'a, 'b> NewBook<'a> where 'b: 'a {
    pub fn from(book: &'b Book) -> Self {
        Self {
            isbn: book.isbn(),
            publisher_id: book.publisher_id() as i64,
            series_id: book.series_id().map(|id| id as i64),
            title: book.title(),
            scheduled_pub_date: book.scheduled_pub_date(),
            actual_pub_date: book.actual_pub_date(),
            registered_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = schema::books::book)]
pub struct BookForm<'a> {
    pub series_id: Option<i64>,
    pub title: &'a str,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,
    pub modified_at: chrono::NaiveDateTime
}

impl <'a, 'b> BookForm<'a> where 'b: 'a {
    pub fn from(book: &'b Book) -> Self {
        Self {
            series_id: book.series_id().map(|id| id as i64),
            title: book.title(),
            scheduled_pub_date: book.scheduled_pub_date(),
            actual_pub_date: book.actual_pub_date(),
            modified_at: chrono::Local::now().naive_local(),
        }
    }
}

pub struct BookPgStore {
    pool: Pool<ConnectionManager<PgConnection>>
}

impl BookPgStore {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

impl BookPgStore {

    pub fn find_by_pub_between(&self, from: &chrono::NaiveDate, to: &chrono::NaiveDate) -> Result<Vec<BookEntity>, Error> {
        use schema::books::book::dsl::*;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;
        let results = book
            .filter(
                actual_pub_date.between(from, to).or(scheduled_pub_date.between(from, to))
            )
            .order_by(id.asc())
            .select(BookEntity::as_select())
            .load(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(results)
    }

    pub fn find_by_isbn(&self, isbn: &[&str]) -> Result<Vec<BookEntity>, Error> {
        use schema::books::book::dsl::{book, id};
        use schema::books::book::dsl::isbn as db_isbn;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;
        let results = book
            .filter(db_isbn.eq_any(isbn))
            .order_by(id.asc())
            .select(BookEntity::as_select())
            .load(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(results)
    }

    pub fn save_books(&self, books: &[Book]) -> Result<Vec<BookEntity>, Error> {
        use schema::books::book;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;

        let entities = books.into_iter()
            .map(|b| NewBook::from(b))
            .collect::<Vec<_>>();

        let results = diesel::insert_into(book::table)
            .values(entities)
            .returning(BookEntity::as_select())
            .get_results(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(results)
    }

    pub fn update_book(&self, book: &Book) -> Result<usize, Error> {
        use schema::books::book;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;
        let updated_count = diesel::update(book::table)
            .set(BookForm::from(book))
            .execute(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(updated_count)
    }
}