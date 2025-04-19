use chrono;
use chrono::NaiveDate;
use diesel::prelude::*;

/// 출판사 모델
#[derive(Queryable, Selectable, Identifiable, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::publisher)]
pub struct Publisher {
    id: i64,
    name: String,
}

impl Publisher {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = crate::book::schema::publisher_keyword)]
#[diesel(primary_key(publisher_id, keyword))]
#[diesel(belongs_to(Publisher, foreign_key = publisher_id))]
pub struct PublisherKeyword {
    publisher_id: i64,
    keyword: String,
}

impl PublisherKeyword {
    pub fn publisher_id(&self) -> i64 {
        self.publisher_id
    }

    pub fn keyword(&self) -> &str {
        &self.keyword
    }
}

/// 도서 시리즈 모델
pub struct Series {
    id: u64,
    name: String,

    isbn: Option<String>,
}

impl Series {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn isbn(&self) -> &Option<String> {
        &self.isbn
    }
}

/// 도서 모델
pub struct Book {
    id: u64,
    isbn: String,
    title: String,

    // 예정된 출판일
    scheduled_pub_date: Option<NaiveDate>,
    // 실제 출판일
    actual_pub_date: Option<NaiveDate>,

    series_id: u64,
}

impl Book {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn isbn(&self) -> &str {
        &self.isbn
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn scheduled_pub_date(&self) -> Option<NaiveDate> {
        self.scheduled_pub_date
    }

    pub fn actual_pub_date(&self) -> Option<NaiveDate> {
        self.actual_pub_date
    }

    pub fn series_id(&self) -> u64 {
        self.series_id
    }
}