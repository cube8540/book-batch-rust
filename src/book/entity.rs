use crate::book::schema::book::dsl::book;
use crate::book::schema::book::isbn;
use crate::book::schema::publisher::dsl::publisher;
use crate::book::schema::publisher_keyword::dsl::publisher_keyword;
use chrono::NaiveDate;
use diesel::associations::HasTable;
use diesel::{Associations, Connection, ExpressionMethods, Identifiable, PgConnection, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper};

#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::publisher)]
pub struct PublisherEntity {
    pub id: i64,
    pub name: String,
}

/// 출판사 API 검색시 사용할 키워드
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::publisher_keyword)]
#[diesel(primary_key(publisher_id, keyword))]
#[diesel(belongs_to(PublisherEntity, foreign_key = publisher_id))]
pub struct PublisherKeywordEntity {
    pub publisher_id: i64,
    pub keyword: String,
}

pub fn find_publisher_all(conn: &mut PgConnection) -> Vec<(PublisherEntity, Option<PublisherKeywordEntity>)> {
    publisher::table()
        .left_join(publisher_keyword::table())
        .select((PublisherEntity::as_select(), Option::<PublisherKeywordEntity>::as_select()))
        .load::<(PublisherEntity, Option<PublisherKeywordEntity>)>(conn)
        .unwrap()
}

/// 도서 시리즈 모델
#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::series)]
pub struct SeriesEntity {
    pub id: i64,
    pub name: Option<String>,
    pub isbn: Option<String>,
}

/// 도서 모델
#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::book)]
pub struct BookEntity {
    pub id: i64,
    pub isbn: String,
    pub title: String,
    pub publisher_id: i64,
    pub scheduled_pub_date: Option<NaiveDate>,
    pub actual_pub_date: Option<NaiveDate>,
    pub series_id: Option<i64>,
}

pub fn find_book_by_isbn(key: &str, conn: &mut PgConnection) -> Vec<BookEntity> {
    book
        .filter(isbn.eq(key.to_string()))
        .select(BookEntity::as_select())
        .load::<BookEntity>(conn)
        .unwrap_or_default()

}