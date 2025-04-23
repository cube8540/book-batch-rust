use diesel::associations::HasTable;
use diesel::{QueryDsl, SelectableHelper};
use std::any::Any;
use std::collections::HashMap;
use std::ops::Deref;

mod entity;
mod schema;
pub mod repository;

/// 출판사 도메인
#[derive(Debug)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: Vec<String>
}

impl Publisher {
    pub fn new(id: u64, name: String) -> Self {
        let keywords: Vec<String> = Vec::new();
        Publisher{
            id,
            name,
            keywords
        }
    }

    pub fn add_keyword(&mut self, keyword: String) {
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

#[derive(Debug)]
pub struct Series {
    pub id: u64,
    pub name: Option<String>,
    pub isbn: Option<String>,
}

pub type Site = String;
pub type Json = String;

#[derive(Debug)]
pub struct Book {
    pub id: u64,
    pub isbn: String,
    pub publisher_id: u64,
    pub series_id: Option<u64>,
    pub title: String,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,
    pub origin_data: HashMap<Site, Json>,
}

pub trait PublisherRepository {
    fn get_all(&self) -> Vec<Publisher>;
}

pub trait BookRepository {
    fn get_by_isbn(&self, isbn: Vec<&str>) -> Vec<Book>;
}

pub trait SeriesRepository {
    fn get_by_isbn(&self, isbn: Vec<&str>) -> Vec<Series>;

    fn new_series(&self, isbn: Option<&str>, name: Option<&str>) -> Series;
}