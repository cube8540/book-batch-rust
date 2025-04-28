use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod entity;
mod schema;
pub mod repository;

/// 출판사 도메인
#[derive(Debug)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: HashMap<Site, Vec<String>>,
}

impl Publisher {
    pub fn new(id: u64, name: String) -> Self {
        Publisher{
            id,
            name,
            keywords: HashMap::new(),
        }
    }

    pub fn add_keyword(&mut self, site: Site, keyword: String) {
        self.keywords
            .entry(site)
            .or_insert_with(Vec::new)
            .push(keyword);
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keywords(&self, site: Site) -> Vec<String> {
        self.keywords
            .get(&site)
            .unwrap_or(&Vec::new())
            .to_vec()
    }
}

impl PartialEq<Self> for Publisher {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

pub type Site = String;
pub type Json = HashMap<String, String>;

#[derive(Debug, Clone)]
pub struct Book {
    pub id: u64,
    pub isbn: String,
    pub publisher_id: u64,
    pub title: String,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,
    pub origin_data: HashMap<Site, Json>,
}

impl Book {
    pub fn merge(self, update: Book) -> Book {
        let mut merged = Book {
            id: self.id,
            isbn: self.isbn.clone(),
            publisher_id: self.publisher_id,
            title: self.title.clone(),
            scheduled_pub_date: self.scheduled_pub_date,
            actual_pub_date: self.actual_pub_date,
            origin_data: self.origin_data,
        };

        if update.title != self.title {
            merged.title = update.title.clone()
        }

        if let Some(sch) = update.scheduled_pub_date {
            if update.scheduled_pub_date != self.scheduled_pub_date {
                merged.scheduled_pub_date = Some(sch);
            }
        }

        if let Some(act) = update.actual_pub_date {
            if update.actual_pub_date != self.actual_pub_date {
                merged.actual_pub_date = Some(act)
            }
        }

        merged
    }
}

pub trait PublisherRepository {
    fn get_all(&self) -> Vec<Publisher>;
}

pub trait BookRepository {
    fn get_by_isbn(&self, isbn: &Vec<&str>) -> Vec<Book>;

    fn new_books(&self, books: Vec<Book>) -> Vec<Book>;

    fn update_books(&self, books: Vec<Book>) -> Vec<Book>;
}

#[derive(Debug)]
pub enum Operator {
    AND,
    OR,
    NOR,
    NAND
}

impl Operator {
    pub fn from_str(s: &str) -> Option<Operator> {
        match s {
            "AND" => Some(Operator::AND),
            "OR" => Some(Operator::OR),
            "NOR" => Some(Operator::NOR),
            "NAND" => Some(Operator::NAND),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct BookOriginFilter {
    pub id: u64,
    pub name: String,
    pub site: Site,
    pub is_root: bool,
    pub operator: Option<Operator>,
    pub property_name: Option<String>,
    pub regex: Option<String>,
    pub children: Vec<Rc<RefCell<BookOriginFilter>>>,
}

impl BookOriginFilter {
    pub fn validate(&self, origin_data: &Json) -> bool {
        if let Some(operator) = &self.operator {
            match operator {
                Operator::AND => {
                    self.children.iter().all(|child| child.borrow().validate(origin_data))
                }
                Operator::OR => {
                    self.children.iter().any(|child| child.borrow().validate(origin_data))
                }
                Operator::NOR => {
                    self.children.iter().all(|child| !child.borrow().validate(origin_data))
                }
                Operator::NAND => {
                    !self.children.iter().all(|child| child.borrow().validate(origin_data))
                }
            }
        } else if let (Some(regex), Some(property_name)) = (&self.regex, &self.property_name) {
            let regex = Regex::new(regex).unwrap();
            if let Some(value) = origin_data.get(property_name) {
                regex.is_match(value)
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl BookOriginFilter {

    pub fn add_child(&mut self, child: Rc<RefCell<BookOriginFilter>>) {
        self.children.push(child);
    }
}

pub trait BookOriginFilterRepository {
    fn get_root_filters(&self) -> HashMap<Site, Rc<RefCell<BookOriginFilter>>>;
}