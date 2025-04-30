use crate::book;
use crate::book::repository::diesel::schema;
use crate::book::{Book, BookOriginFilter, Original, Site};
use chrono::{NaiveDate, NaiveDateTime};
use diesel::{AsChangeset, Associations, ExpressionMethods, Identifiable, Insertable, Queryable, Selectable};

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
    pub fn to_domain(&self) -> Book {
        Book {
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
    pub scheduled_pub_date: Option<&'a NaiveDate>,
    pub actual_pub_date: Option<&'a NaiveDate>,
    pub registered_at: NaiveDateTime,
}

impl <'a> NewBookEntity<'a> {

    pub fn new(book: &'a Book) -> Self {
        Self {
            isbn: &book.isbn,
            title: &book.title,
            publisher_id: book.publisher_id as i64,
            scheduled_pub_date: book.scheduled_pub_date.as_ref(),
            actual_pub_date: book.actual_pub_date.as_ref(),
            registered_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(AsChangeset)]
#[diesel(table_name = schema::book)]
pub struct BookForm<'a> {
    pub title: &'a str,
    pub scheduled_pub_date: Option<&'a NaiveDate>,
    pub actual_pub_date: Option<&'a NaiveDate>,
    pub modified_at: NaiveDateTime,
}

impl<'a> BookForm<'a> {

    pub fn new(book: &'a Book) -> Self {
        Self {
            title: &book.title,
            scheduled_pub_date: book.scheduled_pub_date.as_ref(),
            actual_pub_date: book.actual_pub_date.as_ref(),
            modified_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Queryable, Selectable, Identifiable, Associations, Debug, PartialEq)]
#[diesel(table_name = schema::book_origin_data)]
#[diesel(primary_key(book_id, site, property))]
#[diesel(belongs_to(BookEntity, foreign_key = book_id))]
pub struct BookOriginDataEntity {
    pub book_id: i64,
    pub site: String,
    pub property: String,
    pub val: Option<String>
}

#[derive(Insertable, Debug, PartialEq)]
#[diesel(table_name = schema::book_origin_data)]
pub struct NewBookOriginDataEntity<'a> {
    pub book_id: i64,
    pub site: &'a str,
    pub property: &'a str,
    pub val: Option<&'a str>,
}

impl <'job, 'a> NewBookOriginDataEntity<'a>
where 'job: 'a {

    pub fn new(id: i64, site: &'job Site, origin: &'job Original) -> Vec<Self> {
        origin.iter()
            .map(|(property, val)| {
                Self {
                    book_id: id,
                    property,
                    site,
                    val: Some(val)
                }
            })
            .collect()
    }
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
    pub regex_val: Option<String>,
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
            regex: self.regex_val.clone(),
            nodes: Vec::new(),
        };
        (filter, self.parent_id.map(|p| p as u64))
    }
}