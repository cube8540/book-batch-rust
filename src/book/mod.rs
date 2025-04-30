use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

mod entity;
mod schema;
pub mod repository;

/// 출판사 도메인
#[derive(Debug, Clone)]
pub struct Publisher {
    pub id: u64,
    pub name: String,
    pub keywords: HashMap<Site, Vec<String>>,
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
}

/// 도서 정보를 가지고 온 사이트
pub type Site = String;

/// 사이트에서 가져온 도서의 원본 데이터
pub type Original = HashMap<String, String>;

/// 도서 정보
#[derive(Debug, Clone)]
pub struct Book {
    pub id: u64,
    pub isbn: String,
    pub publisher_id: u64,
    pub title: String,
    pub scheduled_pub_date: Option<chrono::NaiveDate>,
    pub actual_pub_date: Option<chrono::NaiveDate>,
    pub origin_data: HashMap<Site, Original>,
}

impl Book {
    pub fn merge(&mut self, update: &Book) {
        if update.title != self.title {
            self.title = update.title.clone()
        }

        if let Some(sch) = update.scheduled_pub_date {
            if update.scheduled_pub_date != self.scheduled_pub_date {
                self.scheduled_pub_date = Some(sch);
            }
        }

        if let Some(act) = update.actual_pub_date {
            if update.actual_pub_date != self.actual_pub_date {
                self.actual_pub_date = Some(act)
            }
        }

        for (site, origin) in &update.origin_data {
            let origin_entry = self.origin_data.entry(site.clone())
                .or_insert_with(HashMap::new);
            for (property, value) in origin {
                origin_entry.insert(property.clone(), value.clone());
            }
        }
    }

    pub fn add_origin_data(&mut self, site: Site, property: String, value: String) {
        self.origin_data.entry(site)
            .or_insert_with(HashMap::new)
            .insert(property, value);
    }
}

pub trait PublisherRepository {
    fn get_all(&self) -> Vec<Publisher>;
}

pub trait BookRepository {
    fn get_by_isbn(&self, isbn: &[&str]) -> Vec<Book>;
    
    fn get_book_only_by_isbn(&self, isbn: &[&str]) -> Vec<Book>;

    fn new_books(&self, books: &[&Book]) -> Vec<Book>;

    fn update_books(&self, books: &[&Book]) -> Vec<Book>;
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

pub type Node = Rc<RefCell<BookOriginFilter>>;

/// 도서의 원본 데이터를 이용하여 도서가 유효한지 판단한다.
#[derive(Debug)]
pub struct BookOriginFilter {
    pub id: u64,
    pub name: String,
    pub site: Site,
    pub is_root: bool,
    pub operator: Option<Operator>,
    pub property_name: Option<String>,
    pub regex: Option<String>,
    pub nodes: Vec<Node>,
}

impl BookOriginFilter {

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn validate(&self, origin: &Original) -> bool {
        if let Some(o) = &self.operator {
            return match o {
                Operator::AND => self.nodes.iter().all(|c| c.borrow().validate(origin)),
                Operator::OR => self.nodes.iter().any(|c| c.borrow().validate(origin)),
                Operator::NOR => self.nodes.iter().all(|c| !c.borrow().validate(origin)),
                Operator::NAND => !self.nodes.iter().all(|c| c.borrow().validate(origin))
            }
        }
        if let (Some(regex), Some(property)) = (&self.regex, &self.property_name) {
            let regex = Regex::new(regex).unwrap();
            return origin.get(property).map_or(false, |v| regex.is_match(v))
        }

        false
    }
}

pub trait BookOriginFilterRepository {
    fn get_root_filters(&self) -> HashMap<Site, Node>;
}