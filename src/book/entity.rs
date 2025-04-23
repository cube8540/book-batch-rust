use crate::book::schema;
use chrono::NaiveDate;
use diesel::associations::HasTable;
use diesel::{Associations, Connection, ExpressionMethods, Identifiable, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper};

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = schema::publisher)]
pub struct PublisherEntity {
    pub id: i64,
    pub name: String,
}

/// 출판사 API 검색시 사용할 키워드
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = schema::publisher_keyword)]
#[diesel(primary_key(publisher_id, keyword))]
#[diesel(belongs_to(PublisherEntity, foreign_key = publisher_id))]
pub struct PublisherKeywordEntity {
    pub publisher_id: i64,
    pub keyword: String,
}

pub fn find_publisher_all(conn: &mut PgConnection) -> Vec<(PublisherEntity, Option<PublisherKeywordEntity>)> {
    schema::publisher::dsl::publisher::table()
        .left_join(schema::publisher_keyword::dsl::publisher_keyword::table())
        .select((PublisherEntity::as_select(), Option::<PublisherKeywordEntity>::as_select()))
        .load::<(PublisherEntity, Option<PublisherKeywordEntity>)>(conn)
        .unwrap()
}

/// 도서 시리즈 모델
#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = schema::series)]
pub struct SeriesEntity {
    pub id: i64,
    pub name: Option<String>,
    pub isbn: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = schema::series)]
pub struct NewSeriesEntity<'a> {
    pub isbn: Option<&'a str>,
    pub name: Option<&'a str>,
}

pub fn find_series_by_isbn(conn: &mut PgConnection, isbn: Vec<&str>) -> Vec<SeriesEntity> {
    schema::series::dsl::series
        .filter(schema::series::isbn.eq_any(isbn))
        .select(SeriesEntity::as_select())
        .load(conn)
        .unwrap_or_default()
}

pub fn create_series(conn: &mut PgConnection, isbn: Option<&str>, name: Option<&str>) -> SeriesEntity {
    let new_series = NewSeriesEntity {
        isbn, name
    };

    diesel::insert_into(schema::series::dsl::series::table())
        .values(&new_series)
        .returning(SeriesEntity::as_returning())
        .get_result(conn)
        .expect("Error creating new series")
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
    pub series_id: Option<i64>,
}

pub fn find_book_by_isbn(conn: &mut PgConnection, isbn: Vec<&str>) -> Vec<BookEntity> {
    schema::book::dsl::book
        .filter(schema::book::isbn.eq_any(isbn))
        .select(BookEntity::as_select())
        .load(conn)
        .unwrap_or_default()
}