use std::any::Any;
use diesel::associations::HasTable;
use diesel::{PgConnection, QueryDsl, SelectableHelper};
use std::collections::HashMap;
use std::ops::Deref;

mod entity;
mod schema;

/// 출판사 도메인
#[derive(Debug)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: Vec<String>
}

impl Publisher {
    fn new(id: u64, name: String) -> Self {
        let keywords: Vec<String> = Vec::new();
        Publisher{
            id,
            name,
            keywords
        }
    }

    fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keywords(&self) -> &Vec<String> {
        &self.keywords
    }
}

impl PartialEq<Self> for Publisher {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type Site = String;
pub type Json = HashMap<String, dyn Any>;

pub struct Book {
    pub id: u64,
    pub isbn: String,
    pub publisher_id: u64,
    pub series_id: u64,
    pub title: String,
    pub scheduled_pub_date: chrono::NaiveDate,
    pub actual_pub_date: chrono::NaiveDate,
    pub origin_data: HashMap<Site, Json>,
}

pub fn get_publisher_all(conn: &mut PgConnection) -> Vec<Publisher> {
    let result_set = entity::find_publisher_all(conn);

    let mut map = HashMap::<u64, Publisher>::new();
    result_set.iter().for_each(|item| {
        let publisher_entity = &item.0;
        let keyword_entity = &item.1;

        let id = publisher_entity.id as u64;
        let publisher = map.entry(id)
            .or_insert_with(|| Publisher::new(id, publisher_entity.name.clone()));

        if let Some(k) = keyword_entity {
            publisher.add_keyword(k.keyword.clone())
        }
    });
    map.into_values().collect()
}