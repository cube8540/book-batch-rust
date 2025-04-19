use diesel::associations::HasTable;
use diesel::{QueryDsl, SelectableHelper};
use std::ops::Deref;

pub mod entity;
mod schema;

/// 출판사 도메인
#[derive(Debug)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: Vec<String>
}

impl Publisher {
    fn new(id: u64, n: String) -> Self {
        let keywords: Vec<String> = Vec::new();
        Publisher{
            id,
            name: n,
            keywords
        }
    }

    fn add_keyword(&mut self, keyword: String) {
        self.keywords.push(keyword);
    }
}

impl PartialEq<Self> for Publisher {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}