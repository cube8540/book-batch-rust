use crate::book;
use crate::book::{schema, BookOriginFilter};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{AsChangeset, Associations, ExpressionMethods, Identifiable, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper};

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = schema::publisher)]
pub struct PublisherEntity {
    pub id: i64,
    pub name: String,
}

/// 출판사 API 검색시 사용할 키워드
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = schema::publisher_keyword)]
#[diesel(primary_key(publisher_id, site, keyword))]
#[diesel(belongs_to(PublisherEntity, foreign_key = publisher_id))]
pub struct PublisherKeywordEntity {
    pub publisher_id: i64,
    pub site: String,
    pub keyword: String,
}

pub fn find_publisher_all(conn: &mut PgConnection) -> Vec<(PublisherEntity, Option<PublisherKeywordEntity>)> {
    schema::publisher::table
        .left_join(schema::publisher_keyword::table)
        .select((
            PublisherEntity::as_select(),
            Option::<PublisherKeywordEntity>::as_select()
        ))
        .into_boxed()
        .load(conn)
        .unwrap()
}

/// 도서 모델
#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = schema::book)]
pub struct BookEntity {
    pub id: i64,
    pub isbn: String,
    pub title: String,
    pub publisher_id: i64,
    pub scheduled_pub_date: Option<NaiveDate>,
    pub actual_pub_date: Option<NaiveDate>,
    pub registered_at: NaiveDateTime,
    pub modified_at: Option<NaiveDateTime>,
}

impl BookEntity {
    pub fn to_domain(&self) -> book::Book {
        book::Book {
            id: self.id as u64,
            isbn: self.isbn.clone(),
            publisher_id: self.publisher_id as u64,
            title: self.title.clone(),
            scheduled_pub_date: self.scheduled_pub_date.clone(),
            actual_pub_date: self.actual_pub_date.clone(),
            origin_data: Default::default(),
        }
    }
}

#[derive(Insertable, Debug, PartialEq)]
#[diesel(table_name = schema::book)]
pub struct NewBookEntity<'a> {
    pub isbn: &'a str,
    pub title: &'a str,
    pub publisher_id: i64,
    pub scheduled_pub_date: Option<NaiveDate>,
    pub actual_pub_date: Option<NaiveDate>,
    pub registered_at: NaiveDateTime,
}

#[derive(AsChangeset)]
#[diesel(table_name = schema::book)]
pub struct BookForm<'a> {
    pub title: &'a str,
    pub scheduled_pub_date: Option<&'a NaiveDate>,
    pub actual_pub_date: Option<&'a NaiveDate>,
    pub modified_at: &'a NaiveDateTime,
}

pub fn find_book_by_isbn(conn: &mut PgConnection, isbn: &Vec<&str>) -> Vec<BookEntity> {
    schema::book::table
        .filter(schema::book::isbn.eq_any(isbn))
        .select(BookEntity::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn insert_books(conn: &mut PgConnection, books: Vec<NewBookEntity>) -> Vec<BookEntity> {
    diesel::insert_into(schema::book::table)
        .values(books)
        .get_results(conn)
        .expect("Error inserting new books.")
}

pub fn update_book(conn: &mut PgConnection, isbn: &str, book: BookForm) -> usize {
    diesel::update(schema::book::table)
        .filter(schema::book::isbn.eq(isbn))
        .set(&book)
        .execute(conn)
        .unwrap()
}

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = schema::book_origin_filter)]
pub struct BookOriginFilterEntity {
    pub id: i64,
    pub name: String,
    pub site: String,
    pub is_root: bool,
    pub operator_type: Option<String>,
    pub property_name: Option<String>,
    pub regex: Option<String>,
    pub parent_id: Option<i64>,
}

impl BookOriginFilterEntity {

    pub fn to_domain(self) -> (BookOriginFilter, Option<u64>) {
        let filter = BookOriginFilter {
            id: self.id as u64,
            name: self.name.clone(),
            site: self.site.clone(),
            is_root: self.is_root.clone(),
            operator: if let Some(o) = self.operator_type {
                book::Operator::from_str(&o)
            } else {
                None
            },
            property_name: self.property_name.clone(),
            regex: self.regex.clone(),
            children: Vec::new(),
        };
        (filter, self.parent_id.map(|p| p as u64))
    }
}

pub fn find_book_origin_filter_all(conn: &mut PgConnection) -> Vec<BookOriginFilterEntity> {
    schema::book_origin_filter::table
        .select(BookOriginFilterEntity::as_select())
        .load::<BookOriginFilterEntity>(conn)
        .unwrap()
}