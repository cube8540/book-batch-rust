pub mod repo;
mod raw_impl;

use regex::Regex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use tracing::warn;

/// Item 모듈에서 사용할 에러 열거
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemError {
    /// 필수 데이터가 입력 되지 않음
    RequireArgumentMissing(String),

    /// 알 수 없는 열거형 코드
    UnknownCode(String)
}

impl Display for ItemError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// 도서 데이터의 출처
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Site {
    NLGO,
    Naver,
    Aladin,
    KyoboBook
}

impl TryFrom<&str> for Site {
    type Error = ItemError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "nlgo" => Ok(Site::NLGO),
            "naver" => Ok(Site::Naver),
            "aladin" => Ok(Site::Aladin),
            "kyobo" => Ok(Site::KyoboBook),
            _ => Err(ItemError::UnknownCode(value.to_owned()))
        }
    }
}

impl Display for Site {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Site::NLGO => write!(f, "NLGO"),
            Site::Naver => write!(f, "NAVER"),
            Site::Aladin => write!(f, "ALADIN"),
            Site::KyoboBook => write!(f, "KYOBO"),
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

pub type SharedPublisherRepository = Rc<Box<dyn PublisherRepository>>;

/// 출판사 저장소
pub trait PublisherRepository {

    /// 모든 출판사를 가져온다.
    fn get_all(&self) -> Vec<Publisher>;

    /// 전달 받은 아이디로 출판사를 찾는다.
    fn find_by_id(&self, id: &[u64]) -> Vec<Publisher>;
}

/// 도서 시리즈
#[derive(Debug)]
pub struct Series {
    id: u64,
    title: Option<String>,
    isbn: Option<String>,
    vec: Option<Vec<f32>>,
    registered_at: Option<chrono::NaiveDateTime>,
    modified_at: Option<chrono::NaiveDateTime>
}

impl Series {
    pub fn builder() -> SeriesBuilder {
        SeriesBuilder::new()
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn title(&self) -> &Option<String> {
        &self.title
    }

    pub fn isbn(&self) -> &Option<String> {
        &self.isbn
    }

    pub fn vec(&self) -> &Option<Vec<f32>> {
        &self.vec
    }

    pub fn registered_at(&self) -> Option<chrono::NaiveDateTime> {
        self.registered_at
    }

    pub fn modified_at(&self) -> Option<chrono::NaiveDateTime> {
        self.modified_at
    }
}

impl AsRef<Series> for Series {
    fn as_ref(&self) -> &Series {
        self
    }
}

pub struct SeriesBuilder {
    id: Option<u64>,
    title: Option<String>,
    isbn: Option<String>,
    vec: Option<Vec<f32>>,
    registered_at: Option<chrono::NaiveDateTime>,
    modified_at: Option<chrono::NaiveDateTime>,
}

impl SeriesBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            title: None,
            isbn: None,
            vec: None,
            registered_at: None,
            modified_at: None,
        }
    }

    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    pub fn isbn(mut self, isbn: String) -> Self {
        self.isbn = Some(isbn);
        self
    }

    pub fn vec(mut self, vec: Vec<f32>) -> Self {
        self.vec = Some(vec);
        self
    }

    pub fn registered_at(mut self, registered_at: chrono::NaiveDateTime) -> Self {
        self.registered_at = Some(registered_at);
        self
    }

    pub fn modified_at(mut self, modified_at: chrono::NaiveDateTime) -> Self {
        self.modified_at = Some(modified_at);
        self
    }

    pub fn build(self) -> Result<Series, ItemError> {
        Ok(Series {
            id: self.id.unwrap_or(0),
            title: self.title,
            isbn: self.isbn,
            vec: self.vec,
            registered_at: self.registered_at,
            modified_at: self.modified_at,
        })
    }
}

pub type SharedSeriesRepository = Rc<Box<dyn SeriesRepository>>;

/// 시리즈 저장소
pub trait SeriesRepository {

    /// ISBN 리스트를 받아 해당 ISBN을 가지는 시리즈를 찾는다.
    fn find_by_isbn(&self, isbn: &[&str]) -> Vec<Series>;

    /// 전달 받은 시리즈의 백터([`Series::vec`])와 가장 유사한 시리즈를 limit 개수 만큼 찾는다.
    ///
    /// 결과는 튜플로 (유사 시리즈 - 유사도)로 묶여 반환된다.
    fn similarity(&self, series: &Series, limit: i32) -> Vec<(Series, Option<f64>)>;

    /// 전달 받은 시리즈들을 저장소에 저장한다.
    fn new_series(&self, series: &[Series]) -> Vec<Series>;

    /// 전달 받은 시리즈의 `ISBN`을 업데이트 한다.
    fn update_series_isbn(&self, series_id: u64, isbn: &str) -> usize;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RawValue {
    Null,

    Text(String),

    Number(RawNumber),

    Bool(bool),

    Object(HashMap<String, RawValue>),

    Array(Vec<RawValue>),
}

#[derive(Debug, Copy, Clone)]
pub enum RawNumber {
    Undefined,

    UnsignedInt(u64),

    SignedInt(i64),

    Float(f64),
}

pub type Raw = HashMap<String, RawValue>;

/// 도서의 원본 데이터 타입
/// 각 사이트에서 얻어온 실제 데이터를 저장 할 때 사용한다.
pub type Originals = HashMap<Site, Raw>;

/// 도서
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Book {
    id: u64,
    isbn: String,
    publisher_id: u64,
    series_id: Option<u64>,
    title: String,
    scheduled_pub_date: Option<chrono::NaiveDate>,
    actual_pub_date: Option<chrono::NaiveDate>,
    originals: Originals,
    registered_at : Option<chrono::NaiveDateTime>,
    modified_at: Option<chrono::NaiveDateTime>,
}

impl Book {
    pub fn builder() -> BookBuilder {
        BookBuilder::new()
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

    pub fn series_id(&self) -> Option<u64> {
        self.series_id
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

    pub fn registered_at(&self) -> Option<chrono::NaiveDateTime> {
        self.registered_at
    }

    pub fn modified_at(&self) -> Option<chrono::NaiveDateTime> {
        self.modified_at
    }

    pub fn merge(&self, other: &Book) -> Book {
        let mut new_builder = Self::builder()
            .id(self.id)
            .title(self.title.clone())
            .isbn(self.isbn.clone())
            .publisher_id(self.publisher_id);

        for (site, raw) in &self.originals {
            new_builder = new_builder.add_original(site.clone(), raw.clone());
        }

        if self.title != other.title {
            new_builder = new_builder.title(other.title.clone());
        }

        if let Some(spd) = other.scheduled_pub_date {
            if Some(spd) != self.scheduled_pub_date {
                new_builder = new_builder.scheduled_pub_date(spd);
            }
        }

        if let Some(apd) = other.actual_pub_date {
            if Some(apd) != self.actual_pub_date {
                new_builder = new_builder.actual_pub_date(apd);
            }
        }

        for (site, raw) in &other.originals {
            new_builder = new_builder.add_original(site.clone(), raw.clone());
        }

        new_builder.build().unwrap()
    }

    pub fn to_builder(&self) -> BookBuilder {
        let mut builder = BookBuilder::new()
            .id(self.id)
            .isbn(self.isbn.clone())
            .publisher_id(self.publisher_id)
            .title(self.title.clone());

        // series_id가 있는 경우 추가
        if let Some(series_id) = self.series_id {
            builder = builder.series_id(series_id);
        }

        // scheduled_pub_date가 있는 경우 추가
        if let Some(scheduled_date) = self.scheduled_pub_date {
            builder = builder.scheduled_pub_date(scheduled_date);
        }

        // actual_pub_date가 있는 경우 추가
        if let Some(actual_date) = self.actual_pub_date {
            builder = builder.actual_pub_date(actual_date);
        }

        // registered_at이 있는 경우 추가
        if let Some(registered_at) = self.registered_at {
            builder = builder.registered_at(registered_at);
        }

        // modified_at이 있는 경우 추가
        if let Some(modified_at) = self.modified_at {
            builder = builder.modified_at(modified_at);
        }

        // originals 데이터 추가
        for (site, raw) in &self.originals {
            builder = builder.add_original(*site, raw.clone());
        }

        builder
    }
}

impl AsRef<Book> for Book {
    fn as_ref(&self) -> &Book {
        self
    }
}

/// Book 빌더
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BookBuilder {
    id: Option<u64>,
    isbn: Option<String>,
    publisher_id: Option<u64>,
    series_id: Option<u64>,
    title: Option<String>,
    scheduled_pub_date: Option<chrono::NaiveDate>,
    actual_pub_date: Option<chrono::NaiveDate>,
    originals: Originals,
    registered_at: Option<chrono::NaiveDateTime>,
    modified_at: Option<chrono::NaiveDateTime>,
}

impl BookBuilder {
    pub fn new() -> Self {
        Self {
            id: None,
            isbn: None,
            publisher_id: None,
            series_id: None,
            title: None,
            scheduled_pub_date: None,
            actual_pub_date: None,
            originals: HashMap::new(),
            registered_at: None,
            modified_at: None,
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
    
    pub fn add_original_raw(mut self, site: Site, key: &str, raw_value: RawValue) -> Self {
        let raw = self.originals.entry(site)
            .or_insert_with(HashMap::new);
        raw.insert(key.to_owned(), raw_value);
        self
    }

    pub fn series_id(mut self, series_id: u64) -> Self {
        self.series_id = Some(series_id);
        self
    }

    pub fn registered_at(mut self, registered_at: chrono::NaiveDateTime) -> Self {
        self.registered_at = Some(registered_at);
        self
    }

    pub fn modified_at(mut self, modified_at: chrono::NaiveDateTime) -> Self {
        self.modified_at = Some(modified_at);
        self
    }

    pub fn build(self) -> Result<Book, ItemError> {
        let isbn = self.isbn.ok_or(ItemError::RequireArgumentMissing("isbn".to_owned()))?;
        let title = self.title.ok_or(ItemError::RequireArgumentMissing("title".to_owned()))?;

        Ok(Book {
            id: self.id.unwrap_or(0),
            isbn,
            publisher_id: self.publisher_id.unwrap_or(0),
            series_id: self.series_id,
            title,
            scheduled_pub_date: self.scheduled_pub_date,
            actual_pub_date: self.actual_pub_date,
            originals: self.originals,
            registered_at: self.registered_at,
            modified_at: self.modified_at,
        })
    }
}

pub type SharedBookRepository = Rc<Box<dyn BookRepository>>;

/// 도서 저장소
pub trait BookRepository {

    /// 시작 - 종료 날짜를 받아 해당 날짜에 출판 예정이거나, 출판된 도서를 검색한다.
    fn find_by_pub_between(&self, from: &chrono::NaiveDate, to: &chrono::NaiveDate) -> Vec<Book>;

    /// ISBN 리스트를 받아 해당 ISBN을 가진 도서를 찾는다.
    fn find_by_isbn(&self, isbn: &[&str]) -> Vec<Book>;

    /// 전달 받은 도서를 모두 저장소에 저장한다.
    fn save_books(&self, books: &[Book]) -> Vec<Book>;

    /// 전달 받은 도서 정보로 저장소의 도서를 업데이트 한다.
    fn update_book(&self, book: &Book) -> usize;

    /// 시리즈화 되지 않은(시리즈 설정이 되지 않은) 도서를 limit 개수만큼 찾는다.
    fn find_series_unorganized(&self, limit: usize) -> Vec<Book>;

    /// 전달 받은 시리즈로 설정된 도서를 찾는다.
    fn find_by_series_id(&self, series_id: u64) -> Vec<Book>;
}

/// 유효성 체크에 사용할 연산자 열거
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Operator {
    AND,
    OR,
    NOR,
    NAND
}

impl Operator {
    pub fn from_str(v: &str) -> Result<Self, ItemError> {
        match v {
            "AND" => Ok(Operator::AND),
            "OR" => Ok(Operator::OR),
            "NOR" => Ok(Operator::NOR),
            "NAND" => Ok(Operator::NAND),
            _ => Err(ItemError::UnknownCode(format!("Unknown operator: {}", v)))
        }
    }
}

/// 원본 데이터 유효성 검증 피연산자 트레이트
pub trait Operand {
    fn test(&self, raw: &Raw) -> bool;
}

impl <T> Operand for T where T: Fn(&Raw) -> bool {
    fn test(&self, raw: &Raw) -> bool {
        self(raw)
    }
}

/// 원본 데이터 유효성 검증의 연산식 가지고 있는 연산자로 피연산자들을 연산한다.
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use book_batch_rust::item::{Expression, Operand, Operator, Raw, RawNumber, RawValue};
///
/// let raw: Raw = HashMap::from([(String::from("test"), RawValue::from("test"))]);
///
/// let operand1 = Box::new(|raw: &Raw| raw.get("test").is_some());
/// let operand2 = Box::new(|raw: &Raw| raw.get("test").map(|v| v.eq(&RawValue::from("test"))).unwrap_or(false));
///
/// let and_expression = Expression::new(Operator::AND, vec![operand1, operand2]);
/// assert!(and_expression.test(&raw));
/// ```
pub struct Expression(Operator, Vec<Box<dyn Operand>>);

impl Expression {
    pub fn new(op: Operator, operands: Vec<Box<dyn Operand>>) -> Self {
        Self(op, operands)
    }
}

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

/// 도서 원본 데이터 필터 규칙
/// 원본 데이터의 검증 방식을 가지고 있으며 [`FilterRule::to_predicate`]를 통해 피연산자를 변환하여 도서의 유효성 검증을 할 수 있다.
///
/// [`FilterRule`]은 아래와 같이 두 가지 타입으로 구분 된다.
///
/// ## 피연산자
/// 연산자와, 피연산자 목록은 [`None`], 피연산 규칙을 가지고 있을 경우 피연산자로 구분된다.
/// 피연산자는 자신이 가진 규칙을 이용해 실제 원본 데이터의 유효성 검증을 한다.
///
/// ### Example
/// ```
/// use std::collections::HashMap;
/// use regex::Regex;
/// use book_batch_rust::item::{FilterRule, Raw, RawValue};
///
/// let raw: Raw = HashMap::from([(String::from("test"), RawValue::from("1234"))]);
/// let rule = FilterRule::new_operand("연산자 테스트", "test", Regex::new("[0-9]").unwrap());
/// let operand = rule.to_predicate();
///
/// assert!(operand.test(&raw))
/// ```
///
/// ## 연산식
/// 피연산 규칙이 [`None`], 연산자와 피연산자 목록을 가지고 있으면 연산식으로 구분된다.
/// 연산식은 자신이 가지고 있는 피연산자 목록을 이용하여 원본 데이터의 유효성 검사를 하고, 그렇게 얻은 bool 값들을 연산자를 이용해 유효성 검증을 한다.
///
/// ### Example
/// ```
/// use std::cell::RefCell;
/// use std::collections::HashMap;
/// use std::rc::Rc;
/// use regex::Regex;
/// use book_batch_rust::item::{FilterRule, Operator, Raw, RawValue};
///
/// let raw: Raw = HashMap::from([
///     (String::from("first"), RawValue::from("1234")),
///     (String::from("second"), RawValue::from("abcd"))
/// ]);
///
/// let first_rule = FilterRule::new_operand("first rule", "first", Regex::new("[0-9]").unwrap());
/// let second_rule = FilterRule::new_operand("second rule", "second", Regex::new("^[a-zA-Z]+$").unwrap());
///
/// let mut rule = FilterRule::new_operator("연산식 테스트", Operator::AND);
/// rule.add_operand(Rc::new(RefCell::new(first_rule)));
/// rule.add_operand(Rc::new(RefCell::new(second_rule)));
///
/// let operand = rule.to_predicate();
/// assert!(operand.test(&raw));
/// ```
#[derive(Debug, Clone)]
pub struct FilterRule {
    name: String,

    // 연산자
    operator: Option<Operator>,
    // 피연산 규칙
    rule: Option<(String, Regex)>,

    // 연산자 목록
    operands: Vec<Rc<RefCell<FilterRule>>>
}

impl FilterRule {

    pub fn new_operand(name: &str, property_name: &str, regex: Regex) -> Self {
        Self {
            name: name.to_owned(),
            operator: None,
            rule: Some((property_name.to_owned(), regex)),
            operands: Vec::new()
        }
    }

    pub fn new_operator(name: &str, operator: Operator) -> Self {
        Self {
            name: name.to_owned(),
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

    pub fn operands(&self) -> &Vec<Rc<RefCell<FilterRule>>> {
        &self.operands
    }

    pub fn add_operand(&mut self, operand: Rc<RefCell<FilterRule>>) {
        self.operands.push(operand);
    }
}

impl FilterRule {

    pub fn to_predicate(&self) -> Box<dyn Operand> {
        if let Some(operator) = self.operator {
            let operands = self.operands.iter()
                .map(|o| o.borrow().to_predicate())
                .collect();
            Box::new(Expression(operator, operands))
        } else if let Some((property_name, regex)) = self.rule.as_ref() {
            let (property_name, regex) = (property_name.clone(), regex.clone());
            let operand = move |raw: &Raw| {
                let value = raw.get(&property_name).unwrap();
                match value {
                    RawValue::Text(s) => regex.is_match(s),
                    _ => {
                        warn!("Text 타입 이외의 다른 타입은 정규표현식 검사를 할 수 없습니다. {}", value);
                        false
                    }
                }
            };
            Box::new(operand)
        } else {
            Box::new(|_: &Raw| true)
        }
    }
}

pub type SharedFilterRepository = Rc<Box<dyn FilterRepository>>;

/// 필터 저장소
pub trait FilterRepository {

    /// 특정 사이트의 데이터를 필터링하는 규칙을 찾는다.
    fn find_by_site(&self, site: &Site) -> Vec<FilterRule>;
}