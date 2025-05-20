use regex::Regex;
use std::collections::HashMap;

/// Item 모듈에서 사용할 에러 열거
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ItemError {
    /// 필수 데이터가 입력 되지 않음
    RequireArgumentMissing(String),

    /// 알 수 없는 코드
    UnknownCode(String)
}

/// 도서 데이터의 출처
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Site {
    NLGO,
    Naver,
    Aladin,
    KyoboBook
}

impl Site {

    pub fn from_str(code: &str) -> Result<Self, ItemError> {
        match code {
            "nlgo" => Ok(Site::NLGO),
            "naver" => Ok(Site::Naver),
            "aladin" => Ok(Site::Aladin),
            "kyobo" => Ok(Site::KyoboBook),
            _ => Err(ItemError::UnknownCode(code.to_owned()))
        }
    }

    pub fn to_code_str(&self) -> String {
        match self {
            Site::NLGO => "nlgo".to_owned(),
            Site::Naver => "naver".to_owned(),
            Site::Aladin => "aladin".to_owned(),
            Site::KyoboBook => "kyobo".to_owned()
        }
    }
}

/// 출판사
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Publisher {
    id: u64,
    name: String,
    keywords: HashMap<Site, Vec<String>>
}

impl Publisher {

    pub fn new(id: u64, name: String, keywords: HashMap<Site, Vec<String>>) -> Self {
        Self { id, name, keywords }
    }

    pub fn without_keywords(id: u64, name: String) -> Self {
        Self::new(id, name, HashMap::new())
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

    pub fn add_keyword(&mut self, site: Site, keyword: String) {
        self.keywords.entry(site).or_insert_with(Vec::new).push(keyword);
    }
}

/// 출판사 저장소
pub trait PublisherRepository {

    /// 모든 출판사를 가져온다.
    fn get_all(&self) -> Vec<Publisher>;
}

pub type Raw = HashMap<String, String>;

/// 도서의 원본 데이터 타입
/// 각 사이트에서 얻어온 실제 데이터를 저장 할 때 사용한다.
pub type Originals = HashMap<Site, Raw>;

/// 도서
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Book {
    id: u64,
    isbn: String,
    publisher_id: u64,
    title: String,
    scheduled_pub_date: Option<chrono::NaiveDate>,
    actual_pub_date: Option<chrono::NaiveDate>,
    originals: Originals
}

impl Book {
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

    pub fn scheduled_pub_date(&self) -> Option<chrono::NaiveDate> {
        self.scheduled_pub_date
    }

    pub fn actual_pub_date(&self) -> Option<chrono::NaiveDate> {
        self.actual_pub_date
    }

    pub fn originals(&self) -> &Originals {
        &self.originals
    }
}

/// Book 빌더
pub struct BookBuilder {
    id: Option<u64>,
    isbn: Option<String>,
    publisher_id: Option<u64>,
    title: Option<String>,
    scheduled_pub_date: Option<chrono::NaiveDate>,
    actual_pub_date: Option<chrono::NaiveDate>,
    originals: Originals,
}

impl BookBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            isbn: None,
            publisher_id: None,
            title: None,
            scheduled_pub_date: None,
            actual_pub_date: None,
            originals: HashMap::new(),
        }
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn isbn(mut self, isbn: String) -> Self {
        self.isbn = Some(isbn);
        self
    }

    pub fn publisher_id(mut self, publisher_id: u64) -> Self {
        self.publisher_id = Some(publisher_id);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn scheduled_pub_date(mut self, date: chrono::NaiveDate) -> Self {
        self.scheduled_pub_date = Some(date);
        self
    }

    pub fn actual_pub_date(mut self, date: chrono::NaiveDate) -> Self {
        self.actual_pub_date = Some(date);
        self
    }

    pub fn add_original(mut self, site: Site, raw: Raw) -> Self {
        self.originals.insert(site, raw);
        self
    }

    pub fn build(self) -> Result<Book, ItemError> {
        let isbn = self.isbn.ok_or(ItemError::RequireArgumentMissing("isbn".to_owned()))?;
        let publisher_id = self.publisher_id.ok_or(ItemError::RequireArgumentMissing("publisher_id".to_owned()))?;
        let title = self.title.ok_or(ItemError::RequireArgumentMissing("title".to_owned()))?;

        Ok(Book {
            id: self.id.unwrap_or(0),
            isbn,
            publisher_id,
            title,
            scheduled_pub_date: self.scheduled_pub_date,
            actual_pub_date: self.actual_pub_date,
            originals: self.originals,
        })
    }
}

/// 도서 저장소
pub trait BookRepository {

    /// 시작 - 종료 날짜를 받아 해당 날짜에 출판 예정이거나, 출판된 도서를 검색한다.
    fn find_by_pub_between(&self, from: &chrono::NaiveDate, to: &chrono::NaiveDate) -> Vec<Book>;

    /// ISBN 리스트를 받아 해당 ISBN을 가진 도서를 찾는다.
    fn find_by_isbn(&self, isbn: &[&str]) -> Vec<Book>;

    /// 전달 받은 도서를 모두 저장소에 저장한다.
    fn save_books(&self, books: &[Book]) -> Result<(), ()>;

    /// 전달 받은 데이터로 도서 정보를 업데이트 한다.
    ///
    /// 업데이트 될 도서는 [`Book::id`]로 정해진다.
    fn update_book(&self, book: &Book) -> Result<(), ()>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Operator {
    AND,
    OR,
    NOR,
    NAND
}

pub trait Operand {
    fn test(&self, raw: &Raw) -> bool;
}

impl <T> Operand for T where T: Fn(&Raw) -> bool {
    fn test(&self, raw: &Raw) -> bool {
        self(raw)
    }
}

pub struct Expression(Operator, Vec<Box<dyn Operand>>);

impl Operand for Expression {

    fn test(&self, raw: &Raw) -> bool {
        let (op, operands) = (&self.0, &self.1);
        match op {
            Operator::AND => operands.iter().all(|o| o.test(raw)),
            Operator::OR => operands.iter().any(|o| o.test(raw)),
            Operator::NOR => operands.iter().all(|o| !o.test(raw)),
            Operator::NAND => !operands.iter().all(|o| o.test(raw))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    name: String,

    operator: Option<Operator>,
    rule: Option<(String, Regex)>,

    operands: Vec<Filter>
}

impl Filter {

    pub fn new_operand(name: String, property_name: String, regex: Regex) -> Self {
        Self {
            name,
            operator: None,
            rule: Some((property_name, regex)),
            operands: Vec::new()
        }
    }

    pub fn new_operator(name: String, operator: Operator) -> Self {
        Self {
            name,
            operator: Some(operator),
            rule: None,
            operands: Vec::new()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn operator(&self) -> Option<Operator> {
        self.operator
    }

    pub fn rule(&self) -> &Option<(String, Regex)> {
        &self.rule
    }

    pub fn operands(&self) -> &Vec<Filter> {
        &self.operands
    }

    pub fn add_operand(&mut self, operand: Filter) {
        self.operands.push(operand);
    }
}

impl Filter {

    pub fn to_predicate(&self) -> Box<dyn Operand> {
        if let Some(operator) = self.operator {
            let operands = self.operands.iter()
                .map(|o| o.to_predicate())
                .collect();
            Box::new(Expression(operator, operands))
        } else if let Some((property_name, regex)) = self.rule.as_ref() {
            let (property_name, regex) = (property_name.clone(), regex.clone());
            let operand = move |raw: &Raw| {
                let value = raw.get(&property_name).unwrap();
                regex.is_match(value)
            };
            Box::new(operand)
        } else {
            Box::new(|_: &Raw| true)
        }
    }
}

/// 필터 저장소
pub trait FilterRepository {

    /// 모든 필터 정보를 가져온다.
    fn find_filters(&self) -> Vec<(Site, Filter)>;
}