use chrono::NaiveDate;
use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tracing::{debug, enabled, warn};

pub mod repository;

/// 출판사 도메인
#[derive(Debug, Clone)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: HashMap<Site, Vec<String>>,
}

impl Publisher {
    pub fn new(id: u64, name: String) -> Self {
        Self {
            id,
            name,
            keywords: HashMap::new(),
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn keywords(&self) -> &HashMap<Site, Vec<String>> {
        &self.keywords
    }
}

impl Publisher {
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
    id: u64,
    isbn: String,
    publisher_id: u64,
    title: String,
    scheduled_pub_date: Option<NaiveDate>,
    actual_pub_date: Option<NaiveDate>,
    origin_data: HashMap<Site, Original>,
}

impl Book {
    pub fn new(
        isbn: String,
        publisher_id: u64,
        title: String,
        scheduled_pub_date: Option<NaiveDate>,
        actual_pub_date: Option<NaiveDate>,
        origin_data: HashMap<Site, Original>,
    ) -> Self {
        Self {
            id: 0,
            isbn,
            publisher_id,
            title,
            scheduled_pub_date,
            actual_pub_date,
            origin_data,
        }
    }
    
    pub fn exists(
        id: u64,
        isbn: String,
        publisher_id: u64,
        title: String,
        scheduled_pub_date: Option<NaiveDate>,
        actual_pub_date: Option<NaiveDate>,
        origin_data: HashMap<Site, Original>,
    ) -> Self {
        Self {
            id,
            isbn,
            publisher_id,
            title,
            scheduled_pub_date,
            actual_pub_date,
            origin_data,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn isbn(&self) -> &str {
        &self.isbn
    }

    pub fn publisher_id(&self) -> u64 {
        self.publisher_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn scheduled_pub_date(&self) -> Option<&NaiveDate> {
        self.scheduled_pub_date.as_ref()
    }

    pub fn actual_pub_date(&self) -> Option<&NaiveDate> {
        self.actual_pub_date.as_ref()
    }

    pub fn origin_data(&self) -> &HashMap<Site, Original> {
        &self.origin_data
    }

    pub fn set_publisher_id(&mut self, publisher_id: u64) {
        self.publisher_id = publisher_id;
    }
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
    id: u64,
    name: String,
    site: Site,
    is_root: bool,
    operator: Option<Operator>,
    property_name: Option<String>,
    regex: Option<String>,
    nodes: Vec<Node>,
}

impl BookOriginFilter {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn site(&self) -> &str {
        &self.site
    }

    pub fn is_root(&self) -> bool {
        self.is_root
    }

    pub fn operator(&self) -> &Option<Operator> {
        &self.operator
    }

    pub fn property_name(&self) -> &Option<String> {
        &self.property_name
    }

    pub fn regex(&self) -> &Option<String> {
        &self.regex
    }

    pub fn nodes(&self) -> &Vec<Node> {
        &self.nodes
    }
}

impl BookOriginFilter {

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    pub fn validate(&self, book: &Book) -> bool {
        if let Some(origin) = book.origin_data.get(&self.site) {
            if let Some(o) = &self.operator {
                return match o {
                    Operator::AND => self.nodes.iter().all(|c| c.borrow().validate(book)),
                    Operator::OR => self.nodes.iter().any(|c| c.borrow().validate(book)),
                    Operator::NOR => self.nodes.iter().all(|c| !c.borrow().validate(book)),
                    Operator::NAND => !self.nodes.iter().all(|c| c.borrow().validate(book))
                }
            }
            if let (Some(regex), Some(property)) = (&self.regex, &self.property_name) {
                let regex = Regex::new(regex).unwrap();

                let result =  origin.get(property).map_or(false, |v| regex.is_match(v));
                if !result && enabled!(tracing::Level::DEBUG) {
                    debug!("{}에서 얻어온 {}가 {}와 매칭 되지 않아 도서가 필터링 됩니다. (ISBN: {}, 실제 값: {})",
                        self.site, property, regex.as_str(), book.isbn, origin.get(property).unwrap_or(&"".to_string()));
                }
                result
            } else {
                if enabled!(tracing::Level::WARN) {
                    warn!("{}에서 필터링을 위한 프로퍼티/정규식이 입력 되어 있지 않습니다. (ISBN: {})", self.site, book.isbn);
                }
                false
            }
        } else {
            true
        }
    }
}