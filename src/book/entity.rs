use crate::book::schema::publisher::dsl::publisher;
use crate::book::schema::publisher_keyword::dsl::publisher_keyword;
use crate::book::Publisher;

use chrono::NaiveDate;
use diesel::associations::HasTable;
use diesel::{Associations, Identifiable, PgConnection, QueryDsl, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use std::collections::HashMap;

/// 출판사 모델
#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::publisher)]
pub struct PublisherEntity {
    id: i64,
    name: String,
}

/// 출판사 API 검색시 사용할 키워드
#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::publisher_keyword)]
#[diesel(primary_key(publisher_id, keyword))]
#[diesel(belongs_to(Publisher, foreign_key = publisher_id))]
struct PublisherKeywordEntity {
    publisher_id: i64,
    keyword: String,
}

/// 도서 시리즈 모델
struct SeriesEntity {
    id: u64,
    name: String,

    isbn: Option<String>,
}

/// 도서 모델
struct BookEntity {
    id: u64,
    isbn: String,
    title: String,

    // 예정된 출판일
    scheduled_pub_date: Option<NaiveDate>,
    // 실제 출판일
    actual_pub_date: Option<NaiveDate>,

    series_id: u64,
}

pub fn find_publisher_all(conn: &mut PgConnection) -> Vec<Publisher> {
    let publisher_with_keywords: Vec<(PublisherEntity, Option<PublisherKeywordEntity>)> = publisher::table()
        .left_join(publisher_keyword::table())
        .select((PublisherEntity::as_select(), Option::<PublisherKeywordEntity>::as_select()))
        .load::<(PublisherEntity, Option<PublisherKeywordEntity>)>(conn)
        .unwrap();

    let mut map = HashMap::<u64, Publisher>::new();
    publisher_with_keywords.iter().for_each(|item| {
        let publisher_entity = &item.0;
        let keyword_entity = &item.1;

        let id = publisher_entity.id as u64;
        let publ = map.entry(id)
            .or_insert_with(|| Publisher::new(id, publisher_entity.name.clone()));

        if let Some(k) = keyword_entity {
            publ.add_keyword(k.keyword.clone())
        }
    });

    map.into_values().collect()
}