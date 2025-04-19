use crate::book::schema::publisher::dsl::publisher;
use crate::book::schema::publisher_keyword::dsl::publisher_keyword;

use chrono::NaiveDate;
use diesel::associations::HasTable;
use diesel::{Associations, Identifiable, PgConnection, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper};

/// 출판사 모델
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

/// 도서 시리즈 모델
pub struct SeriesEntity {
    id: u64,
    name: String,

    isbn: Option<String>,
}

/// 도서 모델
pub struct BookEntity {
    id: u64,
    isbn: String,
    title: String,

    // 예정된 출판일
    scheduled_pub_date: Option<NaiveDate>,
    // 실제 출판일
    actual_pub_date: Option<NaiveDate>,

    series_id: u64,
}

pub fn find_publisher_all(conn: &mut PgConnection) -> Vec<(PublisherEntity, Option<PublisherKeywordEntity>)> {
    publisher::table()
        .left_join(publisher_keyword::table())
        .select((PublisherEntity::as_select(), Option::<PublisherKeywordEntity>::as_select()))
        .load::<(PublisherEntity, Option<PublisherKeywordEntity>)>(conn)
        .unwrap()
}