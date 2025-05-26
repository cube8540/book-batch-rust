use crate::item::{Book, BookBuilder, FilterRule, Operator, Site};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;
use regex::Regex;
use std::str::FromStr;

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

impl BookEntity {
    pub fn to_domain_builder(&self) -> BookBuilder {
        let mut builder = Book::builder()
            .id(self.id as u64)
            .isbn(self.isbn.clone())
            .publisher_id(self.publisher_id as u64)
            .title(self.title.clone())
            .registered_at(self.registered_at.clone());

        if let Some(series_id) = self.series_id {
            builder = builder.series_id(series_id as u64);
        }
        if let Some(scheduled_pub_date) = self.scheduled_pub_date {
            builder = builder.scheduled_pub_date(scheduled_pub_date);
        }
        if let Some(actual_pub_date) = self.actual_pub_date {
            builder = builder.actual_pub_date(actual_pub_date);
        }
        if let Some(modified_at) = self.modified_at {
            builder = builder.modified_at(modified_at);
        }
        
        builder
    }
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

    pub fn save_books<T: AsRef<Book>>(&self, books: &[T]) -> Result<Vec<BookEntity>, Error> {
        use schema::books::book;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;

        let entities = books.iter()
            .map(|b| NewBook::from(b.as_ref()))
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
            .filter(book::id.eq(book.id() as i64))
            .set(BookForm::from(book))
            .execute(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(updated_count)
    }

    pub fn find_series_unorganized(&self, limit: usize) -> Result<Vec<BookEntity>, Error> {
        use schema::books::book::dsl::*;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;
        let result = book
            .filter(series_id.is_null())
            .limit(limit as i64)
            .order_by(id.desc())
            .select(BookEntity::as_select())
            .load(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(result)
    }

    pub fn find_by_series_id(&self, series_id: u64) -> Result<Vec<BookEntity>, Error> {
        use schema::books::book::dsl::{book, id};
        use schema::books::book::dsl::series_id as db_series_id;

        let series_id = series_id as i64;
        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;
        let result = book
            .filter(db_series_id.nullable().eq(&series_id))
            .order_by(id.asc())
            .select(BookEntity::as_select())
            .load(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(result)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::books::publisher)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PublisherEntity {
    pub id: i64,
    pub name: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::books::publisher_keyword)]
#[diesel(primary_key(publisher_id, site, keyword))]
#[diesel(belongs_to(PublisherEntity, foreign_key = publisher_id))]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PublisherKeywordEntity {
    pub publisher_id: i64,
    pub site: String,
    pub keyword: String,
}

pub struct PublisherPgStore {
    pool: Pool<ConnectionManager<PgConnection>>
}

impl PublisherPgStore {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

impl PublisherPgStore {
    pub fn find_all(&self) -> Result<Vec<(PublisherEntity, Option<PublisherKeywordEntity>)>, Error> {
        use schema::books::publisher;
        use schema::books::publisher_keyword;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;

        let publisher_with_keywords = publisher::table
            .left_join(publisher_keyword::table)
            .select((
                PublisherEntity::as_select(),
                Option::<PublisherKeywordEntity>::as_select()
            ))
            .load::<(PublisherEntity, Option<PublisherKeywordEntity>)>(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(publisher_with_keywords)
    }

    pub fn find_by_id(&self, id: &[u64]) -> Result<Vec<(PublisherEntity, Option<PublisherKeywordEntity>)>, Error> {
        use schema::books::publisher;
        use schema::books::publisher_keyword;

        let id = id.iter().map(|i| i.clone() as i64).collect::<Vec<_>>();
        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;

        let publisher_with_keywords = publisher::table
            .left_join(publisher_keyword::table)
            .filter(publisher::id.eq_any(&id))
            .select((
                PublisherEntity::as_select(),
                Option::<PublisherKeywordEntity>::as_select()
            ))
            .load::<(PublisherEntity, Option<PublisherKeywordEntity>)>(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(publisher_with_keywords)
    }
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::books::book_origin_filter)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BookOriginFilterEntity {
    pub id: i64,
    pub name: String,
    pub site: String,
    pub is_root: bool,
    pub operator_type: Option<String>,
    pub property_name: Option<String>,
    pub regex_val: Option<String>,
    pub parent_id: Option<i64>,
}

impl BookOriginFilterEntity {

    pub fn is_operand(&self) -> bool {
        self.property_name.is_some() && self.regex_val.is_some()
    }

    pub fn is_operator(&self) -> bool {
        self.operator_type.is_some()
    }

    pub fn to_domain(&self) -> FilterRule {
        match self.is_operator() {
            true => {
                let operator = Operator::from_str(&self.operator_type.as_ref().unwrap()).unwrap();
                FilterRule::new_operator(&self.name, operator)
            }
            false => {
                let regex = Regex::from_str(&self.regex_val.as_ref().unwrap()).unwrap();
                FilterRule::new_operand(
                    &self.name,
                    &self.property_name.as_ref().unwrap(),
                    regex
                )
            }
        }
    }
}

pub struct BookOriginFilterPgStore {
    pool: Pool<ConnectionManager<PgConnection>>
}

impl BookOriginFilterPgStore {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

impl BookOriginFilterPgStore {
    pub fn find_by_site(&self, site: &Site) -> Result<Vec<BookOriginFilterEntity>, Error> {
        use schema::books::book_origin_filter::dsl::book_origin_filter;
        use schema::books::book_origin_filter::dsl::site as db_site;

        let mut connection = self.pool.get()
            .map_err(|e| Error::ConnectError(e.to_string()))?;

        let results = book_origin_filter
            .filter(db_site.eq(site.to_string()))
            .select(BookOriginFilterEntity::as_select())
            .load(&mut connection)
            .map_err(|e| Error::SqlExecuteError(e.to_string()))?;

        Ok(results)
    }
}